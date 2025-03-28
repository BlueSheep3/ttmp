use interprocess::os::windows::named_pipe::pipe_mode::{Bytes, Messages};
use interprocess::os::windows::named_pipe::{PipeListener, PipeListenerOptions, PipeMode};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct FileReader {
	file_list: Arc<Mutex<Vec<PathBuf>>>,
}

impl FileReader {
	pub fn new() -> Self {
		Self {
			file_list: Arc::new(Mutex::new(Vec::new())),
		}
	}

	pub fn drain_file_list(&self) -> Vec<PathBuf> {
		let mut file_list = self.file_list.lock().expect("Failed to lock file list");
		file_list.drain(..).collect()
	}

	pub fn start_receiving(&mut self, pipe_name: &str) {
		let pipe_name = pipe_name.to_owned();
		let file_list = Arc::clone(&self.file_list);

		thread::spawn(move || {
			let listener: PipeListener<Bytes, Messages> = PipeListenerOptions::new()
				.path(&*pipe_name)
				.mode(PipeMode::Messages)
				.create()
				.expect("Failed to create named pipe");

			loop {
				match listener.accept() {
					Ok(connection) => {
						let mut reader = BufReader::new(connection);
						let mut buffer = String::new();

						reader
							.read_to_string(&mut buffer)
							.expect("Failed to read message");

						let path = PathBuf::from(buffer.trim());
						let mut file_list = file_list.lock().expect("Failed to lock file list");
						file_list.push(path);
					}
					Err(e) => {
						eprintln!("Failed to accept client connection: {}", e);
						break;
					}
				}
			}
		});
	}
}
