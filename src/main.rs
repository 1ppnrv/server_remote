mod pty;

use async_std::task;
use axum::{
    Json, Router,
    body::Body,
    extract::{
        ConnectInfo, Path, Request, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, path::Path as DirPath, sync::Arc, time};
use tokio::sync::{
    Mutex,
    mpsc::{self, Sender},
};

use crate::pty::log;

#[derive(Clone)]
struct TermsData {
    running: Arc<Mutex<Vec<u8>>>,
    inputs: Arc<Mutex<Vec<Sender<String>>>>,
    outputs: Arc<Mutex<Vec<Sender<String>>>>,
}

#[tokio::main]
async fn main() {
    let running: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));
    let inputs: Arc<Mutex<Vec<Sender<String>>>> = Arc::new(Mutex::new(vec![]));
    let outputs: Arc<Mutex<Vec<Sender<String>>>> = Arc::new(Mutex::new(vec![]));

    let state = TermsData {
        running,
        inputs,
        outputs,
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/{*arg}", get(get_handler))
        .route("/ws/{id}", get(ws_handler))
        .route("/login/{cookie}", post(login_handler))
        .layer(middleware::from_fn(log_middleware))
        .with_state(state);

    if !DirPath::new("output").exists() {
        fs::create_dir("output").expect("failed to create output dir");
    }

    log("Server started".to_string());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:10001")
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn log_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let method_clone = method.clone();
    let uri = request.uri().to_string();
    let uri_clone = uri.clone();
    let ip;

    if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        ip = addr.ip();
    } else {
        ip = "255.255.255.255".parse().expect("valid IP address");
    }

    task::spawn(async move {
        log(format!("{} | {} {} | incoming\n", ip, method, uri));
    });

    let path = request.uri().path();
    let requires_cookie = matches!(path, "/remote" | "/files" | "/outputhist")
        || path.starts_with("/ws/");
    let has_valid_cookie = request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|cookies| {
            cookies
                .split(';')
                .any(|cookie| cookie.trim() == "mrbap_remote=poklokpondpart" || cookie.trim() == "auth=poklokpondpart")
        });

    let response = if requires_cookie && !has_valid_cookie {
        root().await
    } else {
        next.run(request).await
    };

    let status = response.status();
    task::spawn(async move {
        log(format!(
            "{} | {} {} | {}\n",
            ip, method_clone, uri_clone, status
        ));
    });

    response
}

async fn root() -> Response {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            fs::read_to_string("data/index.html").expect("skill issue"),
        ))
        .expect("Failed to build response")
        .into_response()
}

async fn get_handler(Path(arg): Path<String>) -> Response {
    match arg.as_str() {
        "remote" => Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(
                fs::read_to_string("data/remote.html")
                    .expect("skill issue: pas de fichier remote.html"),
            ))
            .expect("Failed to build response")
            .into_response(),
        "files" => Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(
                fs::read_to_string("data/files.html").expect("skill issue: pas de fichier files"),
            ))
            .expect("Failed to build response")
            .into_response(),

        "outputhist" => {
            let mut hists = String::new();
            let output_files = fs::read_dir("./output/").expect("unable to read output dir");
            let output_files_count = fs::read_dir("./output/")
                .expect("unable to read output dir")
                .count();

            for (mut i, output) in output_files.enumerate() {
                i = output_files_count - i;
                hists.push_str(
                    format!(
                        "{}{}endterm",
                        i,
                        fs::read_to_string(output.unwrap().path()).expect("failed to read output")
                    )
                    .as_str(),
                );
            }
            Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(Body::from(hists))
                .expect("Failed to build response")
                .into_response()
        }

        _ => {
            let path = format!("data/{}", arg);
            // log(path.clone());
            match fs::read(&path) {
                Ok(bytes) => {
                    let content_type = if path.ends_with(".js") {
                        "application/javascript"
                    } else if path.ends_with(".css") {
                        "text/css"
                    } else if path.ends_with(".html") {
                        "text/html"
                    } else if path.ends_with(".png") {
                        "image/png"
                    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
                        "image/jpeg"
                    } else if path.ends_with(".svg") {
                        "image/svg+xml"
                    } else {
                        "application/octet-stream"
                    };

                    Response::builder()
                        .status(200)
                        .header("Content-Type", content_type)
                        .body(Body::from(bytes))
                        .expect("Failed to build response")
                        .into_response()
                }
                Err(_) => {
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header("Content-Type", "text/plain")
                        .body(Body::from(format!("skill issue: {} not found", arg)))
                        .expect("Failed to build response")
                        .into_response()
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Login {
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    login_state: u8,
    cookie_value: String,
}

async fn login_handler(Path(arg): Path<String>, Json(data): Json<Login>) -> impl IntoResponse {
    match arg.as_str() {
        "pwd" => {
            log(format!("{} | /{} | {}\n", "POST", arg, "200"));
            if data.password == "pklk" {
                log("Password OK".to_string());
                Json(LoginResponse {
                    login_state: 1,
                    cookie_value: "poklokpondpart".to_string(),
                })
            } else {
                log("Wrong password".to_string());
                Json(LoginResponse {
                    login_state: 0,
                    cookie_value: "".to_string(),
                })
            }
        }
        "cookie" => {
            log(format!("{} | /{} | {}\n", "POST", arg, "200"));
            if data.password == "poklokpondpart" {
                log("Password OK".to_string());
                Json(LoginResponse {
                    login_state: 1,
                    cookie_value: "".to_string(),
                })
            } else {
                log("Wrong password".to_string());
                Json(LoginResponse {
                    login_state: 0,
                    cookie_value: "".to_string(),
                })
            }
        }

        _ => {
            log(format!("{} | /{} | {}\n", "POST", arg, "404"));
            Json(LoginResponse {
                login_state: 0,
                cookie_value: "".to_string(),
            })
        }
    }
}

async fn ws_handler(
    State(termsdata): State<TermsData>,
    ws: WebSocketUpgrade,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        pty_socket(
            socket,
            id,
            termsdata.running.clone(),
            termsdata.inputs.clone(),
            termsdata.outputs.clone(),
        )
    })
}

async fn pty_socket(
    socket: WebSocket,
    id: usize,
    running: Arc<Mutex<Vec<u8>>>,
    inputs: Arc<Mutex<Vec<Sender<String>>>>,
    outputs: Arc<Mutex<Vec<Sender<String>>>>,
) {
    let (sender_pty, mut receiver_pty, stop_channel) = {
        let mut running_lock = running.lock().await;
        while running_lock.len() <= id {
            running_lock.push(0);
        }
        if running_lock[id] != 1 {
            running_lock[id] = 1;

            let mut output = outputs.lock().await;
            let mut input = inputs.lock().await;

            while output.len() <= id {
                let (s, _) = mpsc::channel(32);
                output.push(s.clone());
                input.push(s);
            }
            let (s_input, r_input) = mpsc::channel(32);
            input[id] = s_input.clone();
            let (s_output, r_output) = mpsc::channel(32);
            output[id] = s_output.clone();
            pty::new_pty(
                running.clone(),
                id,
                r_input,
                s_output.clone(),
                outputs.clone(),
            )
            .await;
            (s_input.clone(), r_output, s_output)
        } else {
            let mut output = outputs.lock().await;
            let input = inputs.lock().await;

            let (s, r) = mpsc::channel(32);
            output[id] = s.clone();
            (input[id].clone(), r, s)
        }
    };

	let title = fs::read_to_string(format!("titles/title{}.txt", id)).expect("unable to read title file");

    let (mut writer, mut receiver) = socket.split();

	writer.send(Message::Text(title.into())).await.expect("unable to send title");

    //output receiving from pty and sending in WS
    tokio::spawn(async move {
        loop {
            let msg = receiver_pty.recv().await;
            match msg {
                Some(text) => {
                    if text == "WS_Stop".to_string() {
                        log(format!("WS {}: Closing", id));
                        break;
                    }
                    match writer.send(Message::Text(text.into())).await {
                        Ok(_) => {}
                        Err(e) => {
                            log(format!("WS {}: Err sending output: {}", id, e));
                            break;
                        }
                    }
                }
                None => {
                    log(format!(
                        "WS {}: Terminal {} reception channel closed",
                        id, id
                    ));
                    writer
                        .send(Message::Text("Terminal reception error".into()))
                        .await
                        .expect(format!("WS {}: unable to send error", id).as_str());
                    break;
                }
            }
            tokio::time::sleep(time::Duration::from_millis(200)).await;
        }
    });

    log(format!("WS {}: Open", id));

    // commands receiving from ws and sending in pty input
    loop {
        match receiver.next().await {
            Some(Ok(Message::Text(text))) => {
                if text != "ping".to_string() {
                    match sender_pty.send(text.to_string()).await {
                        Ok(_) => {}
                        Err(e) => log(format!(
                            "WS {}: Terminal {}: unable to send command: {}",
                            id, id, e
                        )),
                    }
                }
            }

            Some(Ok(Message::Ping(_))) => {}

            _ => {
                log(format!("WS {}: bad msg, ws closed", id));
                stop_channel.send("WS_Stop".to_string()).await.ok();
                break;
            }
        }
    }
}
