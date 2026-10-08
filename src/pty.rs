use async_std::task;
use chrono::Local;
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
    let time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let to_log = format!("[{}] {}\n", time, arg.trim_end());
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
    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect(format!("unable to spawn pair for terminal{}", id).as_str());

    let mut child = if consts::OS == "windows" {
        pair.slave
            .spawn_command(CommandBuilder::new("powershell"))
            .expect(format!("unable to spawn child for terminal{}", id).as_str())
    } else {
        pair.slave
            .spawn_command(CommandBuilder::new("bash"))
            .expect(format!("unable to spawn child for terminal{}", id).as_str())
    };

    let mut writer = pair
        .master
        .take_writer()
        .expect(format!("unable to take writer for terminal{}", id).as_str());
    let mut reader = pair
        .master
        .try_clone_reader()
        .expect(format!("unable to clone reader for terminal{}", id).as_str());

    let running_clone: Arc<Mutex<Vec<u8>>> = running.clone();

    //output reading and sending in queue
    tokio::task::spawn_blocking(move || {
        let mut term_output = "pklk".to_string();
        loop {
            if term_output == "pklk".to_string() {
                let mut buf = [0u8; 1024];

                let n = match reader.read(&mut buf) {
                    Ok(n) => n,
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
                    let output_path = format!("output/output{}.txt", id);

                    if !DirPath::new(output_path.as_str()).exists() {
                        if !DirPath::new("output").exists() {
                            fs::create_dir("output").expect("failed to create output dir");
                        }
                        log(format!("Creating output file for terminal {}", id));
                        let _ = File::create(output_path.as_str());
                    }

                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(format!("output/output{}.txt", id))
                        .expect(format!("failed to open output{} file", id).as_str());
                    file.write_all(term_output_clone.as_bytes()).expect(
                        format!("unable to write output of terminal {} in output file", id)
                            .as_str(),
                    );
                });
            }
			// log("sending output".to_string());
            match output.blocking_send(term_output.clone()) {
                Ok(_) => term_output = "pklk".to_string(),

                Err(_) => {
                    thread::sleep(time::Duration::from_millis(500));
                    log(format!("Terminal {}: trying to retrieve output sender", id));
                    task::block_on(async {
                        let new_output = outputs_list.lock().await;
                        output = new_output[id].clone();
                    });
                }
            }
            thread::sleep(time::Duration::from_millis(100));
        }
    });

    //command input from sender
    tokio::spawn(async move {
		let mut command_full = String::new();
        loop {

			let mut command_full_clone = command_full.clone();
            let command = match input.recv().await {
                Some(cmd) => cmd,
                None => break,
            };

			command_full_clone.push_str(command.as_str());

            if command.contains("\r") || command.contains("\n") {
				log(format!("Terminal {}: {}", id, command_full_clone.clone()));
				command_full_clone = String::new();
			}



			if command_full_clone == "KillTerm\n" {
				log(format!("Terminal {}: Killing", id));
				writer.write(b"\x03").expect(format!("Terminal {}: failed to write in pty input", id).as_str());
				writer.flush().expect(format!("Terminal {}: failed to flush pty input", id).as_str());
				tokio::time::sleep(time::Duration::from_millis(50)).await;
				{
					let mut running = running.lock().await;
					running[id] = 0;
				}
				child.kill().ok();
				break;
			}

            match command.as_str() {
                "" => {}
                command => {
                    writer
                        .write_all(command.as_bytes())
                        .expect(format!("Terminal {}: failed to write in pty input", id).as_str());
                    writer
                        .flush()
                        .expect(format!("Terminal {}: failed to flush pty input", id).as_str());
                }
            }
            tokio::time::sleep(time::Duration::from_millis(100)).await;
			command_full = command_full_clone.clone()
        }
    });

    log(format!("Terminal {}: Ready", id));
}
