use async_std::task;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    env::consts,
    fs::{self, File, OpenOptions},
    io::Read,
    io::Write,
    path::Path as DirPath,
    sync::Arc,
    thread, time,
};
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

pub fn log(arg: String) {
    let time_object = time_format::now_ms().unwrap();
	let time_string = time_format::strftime_ms_utc("%Y-%m-%d %H:%M:%S.{ms}", time_object).unwrap();

    let to_log = format!("[{}] {}\n", time_string, arg.trim_end());
    print!("{}", to_log);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("log.txt")
        .expect("failed to open log file");
    file.write_all(to_log.as_bytes())
        .expect("log function: failed to write in log.txt");
}

pub async fn new_pty(
    running: Arc<Mutex<Vec<u8>>>,
    id: usize,
    mut input: Receiver<String>,
    mut output: Sender<String>,
    outputs_list: Arc<Mutex<Vec<Sender<String>>>>,
) {
    {
        let output_path = format!("output/output{}.txt", id);

        if !DirPath::new(output_path.as_str()).exists() {
            if !DirPath::new("output").exists() {
                fs::create_dir("output").expect("failed to create output dir");
            }
            log(format!("Creating output file for terminal {}", id));
            let _ = File::create(output_path.as_str());
        }

        let output_path = format!("titles/title{}.txt", id);

        if !DirPath::new(output_path.as_str()).exists() {
            if !DirPath::new("titles").exists() {
                fs::create_dir("titles").expect("failed to create output dir");
            }
            log(format!("Creating title file for terminal {}", id));
            let _ = File::create(output_path.as_str());
        }
    }

    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows: 40,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap_or_else(|_| panic!("unable to spawn pair for terminal{}", id));

    let mut _child = if consts::OS == "windows" {
        pair.slave
            .spawn_command(CommandBuilder::new("powershell"))
            .unwrap_or_else(|_| panic!("unable to spawn child for terminal{}", id))
    } else {
        pair.slave
            .spawn_command(CommandBuilder::new("bash"))
            .unwrap_or_else(|_| panic!("unable to spawn child for terminal{}", id))
    };

    let mut writer = pair
        .master
        .take_writer()
        .unwrap_or_else(|_| panic!("unable to take writer for terminal{}", id));
    let mut reader = pair
        .master
        .try_clone_reader()
        .unwrap_or_else(|_| panic!("unable to clone reader for terminal{}", id));

    let running_clone: Arc<Mutex<Vec<u8>>> = running.clone();

    //output reading and sending in queue
    tokio::task::spawn_blocking(move || {
        let mut term_output = "pklk".to_string();
        loop {
            if term_output == "pklk" {
                let mut buf = [0u8; 1024];

                let n = match reader.read(&mut buf) {
                    Ok(n) => {
						n
					},
                    Err(e) => {
                        log(format!(
                            "Terminal {}: Failed to read output buffer: {}",
                            id, e
                        ));
                        task::block_on(async {
                            let mut running = running_clone.lock().await;
                            running[id] = 0;
                        });
                        break;
                    }
                };

                //log(format!("Terminal {}: sending output", id));
                term_output = String::from_utf8_lossy(&buf[..n]).to_string();

                let term_output_clone = term_output.clone();

                task::spawn(async move {
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(format!("output/output{}.txt", id))
                        .unwrap_or_else(|_| panic!("failed to open output{} file", id));
                    file.write_all(term_output_clone.as_bytes())
                        .unwrap_or_else(|_| {
                            panic!("unable to write output of terminal {} in output file", id)
                        });
                });
            }
            // log("sending output".to_string());
            match output.blocking_send(term_output.clone()) {
                Ok(_) => {
					term_output = "pklk".to_string();
				},

                Err(_) => {
                    thread::sleep(time::Duration::from_millis(100));
                    task::block_on(async {
                        let new_output = outputs_list.lock().await;
                        output = new_output[id].clone();
                    });
                }
            }
        }
    });

    //command input from sender
    tokio::spawn(async move {
        loop {
            let command = match input.recv().await {
                Some(cmd) => {
					cmd
				},
                None => break,
            };

            match command.as_str() {
                "" => {}
                command => {
                    writer.write_all(command.as_bytes()).unwrap_or_else(|_| {
                        panic!("Terminal {}: failed to write in pty input", id)
                    });
                    writer
                        .flush()
                        .unwrap_or_else(|_| panic!("Terminal {}: failed to flush pty input", id));
                }
            }

            tokio::time::sleep(time::Duration::from_millis(5)).await;
        }
    });

    log(format!("Terminal {}: Ready", id));
}
