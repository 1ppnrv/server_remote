mod pty;

use axum::{
    Json,
    Router,
    body::Body,
    extract::{
        Path,
        State,
        ws::{
            Message,
            WebSocket,
            WebSocketUpgrade
        },
    },
    http::{StatusCode},
    response::{
        IntoResponse,
        Response
    },
    routing::{get, post}
};
use std::{
    fs, net::SocketAddr, sync::{Arc, OnceLock}, time
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc::{self, Sender}, Mutex},
};
use futures_util::{SinkExt, StreamExt};

#[derive(Clone)]
struct TermsData {
    running: Arc<Mutex<Vec<u8>>>,
    inputs: Arc<Mutex<Vec<Sender<String>>>>,
    outputs: Arc<Mutex<Vec<Sender<String>>>>,
}

static LOG_SENDER:OnceLock<Sender<String>> = OnceLock::new();

#[tokio::main]
async fn main() {

    let running: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));
    let inputs: Arc<Mutex<Vec<Sender<String>>>> = Arc::new(Mutex::new(vec![]));
    let outputs: Arc<Mutex<Vec<Sender<String>>>> = Arc::new(Mutex::new(vec![]));

    let (log_sender, mut _log_receiver) = mpsc::channel(32);
    LOG_SENDER.set(log_sender).expect("unable to set log sender");

    let state = TermsData {
        running,
        inputs,
        outputs,
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/login/*arg", post(login_handler))
        .route("/*arg", get(get_handler))
        .route("/ws/:id", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 10001));

    

    // tokio::spawn(async move {
    //     loop{
    //         match log_receiver.recv().await {
    //             Some(text) =>{
    //                 let mut file = OpenOptions::new()
    //                     .append(true)
    //                     .create(true)
    //                     .open("log.txt")
    //                     .expect("failed to open log file");
    //                 file.write_all(text.as_bytes())
    //                     .expect("log function: failed to write in log.txt");
    //             }
    //             None => {}
    //         }
    //     }
    // });

    pty::log("Server started".to_string());

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Failed to bind server to addr");

}

async fn root() -> Response {
    pty::log(format!("{} | {} | {}\n", "GET", "/", "200"));
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            fs::read_to_string("data/index.html").expect("skill issue"),
        ))
        .expect("Failed to build response")
        .into_response()
}

async fn get_handler(
    State(termsdata): State<TermsData>,
    Path(arg): Path<String>,
) -> Response {

    match arg.as_str() {
        "console" => {
            pty::log(format!("{} | /{} | {}\n", "GET", arg, "200"));
            Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(Body::from(
                    fs::read_to_string("data/console.html").expect("skill issue: pas de fichier console"),
                ))
                .expect("Failed to build response")
                .into_response()
        }
        "files" => {
            pty::log(format!("{} | /{} | {}\n", "GET", arg, "200"));
            Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(Body::from(
                    fs::read_to_string("data/files.html").expect("skill issue: pas de fichier files"),
                ))
                .expect("Failed to build response")
                .into_response()
        }
        

        "hist" => {
            let mut hists = String::new();
            {
                let runnning_lock = termsdata.running.lock().await;
                let nb_running = runnning_lock.len();
                
                for i in 0..nb_running{
                    if runnning_lock[i] == 1{
                        hists.push_str(format!(
                            "{}{}endterm", i, fs::read_to_string(format!("output/output{}.txt", i)).expect("failed to read output"))
                            .as_str()
                        );
                    } else {
                        hists.push_str("OFFendterm");
                    }
                }
            }
            
            pty::log(format!("{} | /{} | {}\n", "GET", arg, "200"));
            Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(Body::from(hists))
                .expect("Failed to build response")
                .into_response()
        }

        _ => {
            let path = format!("data/{}", arg);
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

                    pty::log(format!("{} | /{} | {}\n", "GET", arg, "200"));
                    Response::builder()
                        .status(200)
                        .header("Content-Type", content_type)
                        .body(Body::from(bytes))
                        .expect("Failed to build response")
                        .into_response()
                }
                Err(_) => {
                    pty::log(format!("{} | /{} | {}\n", "GET", arg, "404"));
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

async fn login_handler(
    Path(arg): Path<String>,
    Json(data): Json<Login>,
) -> impl IntoResponse {
    match arg.as_str() {
        "pwd" => {
            pty::log(format!("{} | /{} | {}\n", "POST", arg, "200"));
            if data.password == "pklk" {
                pty::log("Password OK".to_string());
                Json(LoginResponse {
                    login_state: 1,
                    cookie_value: "poklokpondpart".to_string(),
                })
            } else {
                pty::log("Wrong password".to_string());
                Json(LoginResponse {
                    login_state: 0,
                    cookie_value: "".to_string(),
                })
            }
        }
        "cookie" => {
            pty::log(format!("{} | /{} | {}\n", "POST", arg, "200"));
            if data.password == "poklokpondpart" {
                pty::log("Cookie OK".to_string());
                Json(LoginResponse {
                    login_state: 1,
                    cookie_value: "".to_string(),
                })
            } else {
                pty::log("Wrong cookie".to_string());
                Json(LoginResponse {
                    login_state: 0,
                    cookie_value: "".to_string(),
                })
            }
        }

        _ => {
            pty::log(format!("{} | /{} | {}\n", "POST", arg, "404"));
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
    
    ws.on_upgrade(move |socket| pty_socket(socket,
        id,
        termsdata.running.clone(),
        termsdata.inputs.clone(),
        termsdata.outputs.clone()
    ))

}

async fn pty_socket(
    socket: WebSocket,
    id: usize,
    running: Arc<Mutex<Vec<u8>>>,
    inputs: Arc<Mutex<Vec<Sender<String>>>>,
    outputs: Arc<Mutex<Vec<Sender<String>>>>
){
    let (sender_pty, mut receiver_pty, stop_channel) = {
        let mut running_lock = running.lock().await;
        while running_lock.len() <= id{
            running_lock.push(0);
        }
        if running_lock[id] != 1 {
            running_lock[id] = 1;

            let mut output = outputs.lock().await;
            let mut input = inputs.lock().await;

            while output.len() <= id {
                let (s , _) = mpsc::channel(32);
                output.push(s.clone());
                input.push(s);
            }
            let (s_input, r_input) = mpsc::channel(32);
            input[id] = s_input.clone();
            let (s_output, r_output) = mpsc::channel(32);
            output[id] = s_output.clone();
            pty::new_pty(running.clone(), id, r_input, s_output.clone(), outputs.clone()).await;
            (s_input.clone(), r_output, s_output)
            
        } else {
            let mut output = outputs.lock().await;
            let input = inputs.lock().await;
            
            let (s, r) = mpsc::channel(32);
            output[id] = s.clone();
            (input[id].clone(), r, s)
        }
    };

    let (mut writer, mut receiver) = socket.split();

    //output receiving from pty and sending in WS
    tokio::spawn(async move {
        loop{
            let msg = receiver_pty.recv().await;
            match msg {
                Some(text) => {
                    if text == "WS_Stop".to_string(){
                        pty::log(format!("WS {}: Closing", id));
                        break
                    }
                    match writer.send(Message::Text(text)).await {
                        Ok(_) => {},
                        Err(e) => {
                            pty::log(format!("WS {}: Err sending output: {}", id, e));
                            break
                        },
                    }
                }
                None => {
                    pty::log(format!("WS {}: Terminal {} reception channel closed", id, id));
                    writer.send(Message::Text("Terminal reception error".to_string()))
                        .await.expect(format!("WS {}: unable to send error", id).as_str());
                    break
                }
            }
            tokio::time::sleep(time::Duration::from_millis(200)).await;
        }
    });

    pty::log(format!("WS {}: Open", id));

    // commands receiving from ws and sending in pty input
    loop {
        match receiver.next().await {
            Some(Ok(Message::Text(text))) => {
                pty::log(format!("WS {}: {}", id, text.clone()));
                if text != "ping".to_string(){
                    match sender_pty.send(text).await {
                        Ok(_) => {}
                        Err(e) => {
                            pty::log(format!("WS {}: Terminal {}: unable to send command: {}", id, id, e))
                        }
                    }
                }
            }

            Some(Ok(Message::Ping(_))) => {}

            _ => {
                pty::log(format!("WS {}: bad msg, ws closed", id));
                stop_channel.send("WS_Stop".to_string()).await.ok();
                break
                // pty::log(format!("WS {}: msg is not a text , trying to write it in errors.txt", id));
                // let mut file = OpenOptions::new()
                //     .append(true)
                //     .create(true)
                //     .open("errors.txt")
                //     .expect("failed to open log file");
                // file.write_all(
                //     msg
                //     .expect("the receiver.next().await returned an error")
                //     .expect("the ws returned an error")
                //     .into_text()
                //     .expect("failed to convert the error msg to text")
                //     .as_bytes()
                // ).expect(format!("WS {}:failed to write in errors.txt", id).as_str());
            }
        }
    }
}