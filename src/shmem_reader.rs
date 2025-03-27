use interprocess::os::windows::named_pipe::pipe_mode::{Bytes, Messages};
use interprocess::os::windows::named_pipe::{PipeListener, PipeListenerOptions, PipeMode};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct FileReader {
	file_list: Arc<Mutex<Vec<PathBuf>>>,
	listener_thread: Option<thread::JoinHandle<()>>,
}

impl FileReader {
	// Constructor to initialize FileReader
	pub fn new() -> Self {
		Self {
			file_list: Arc::new(Mutex::new(Vec::new())),
			listener_thread: None,
		}
	}

	// Function to drain the file list
	pub fn drain_file_list(&self) -> Vec<PathBuf> {
		let mut file_list = self.file_list.lock().expect("Failed to lock file list");
		file_list.drain(..).collect() // Collect and clear the list
	}

	// Function to start receiving file paths in a separate thread
	pub fn start_receiving(&mut self, pipe_name: &str) {
		let pipe_name = pipe_name.to_owned();
		let file_list = Arc::clone(&self.file_list);

		let handle = thread::spawn(move || {
			// Create the named pipe
			let listener: PipeListener<Bytes, Messages> = PipeListenerOptions::new()
				.path(&*pipe_name)
				.mode(PipeMode::Messages)
				.create()
				.expect("Failed to create named pipe");

			loop {
				// Accept a client connection
				match listener.accept() {
					Ok(connection) => {
						// For each connection, spawn a new thread to handle it
						let file_list = Arc::clone(&file_list);
						thread::spawn(move || {
							let mut reader = BufReader::new(connection);
							let mut buffer = String::new();

							// Read the message sent by the client
							reader
								.read_to_string(&mut buffer)
								.expect("Failed to read message");

							// Convert the string into a PathBuf and store it
							let path = PathBuf::from(buffer.trim());
							let mut file_list = file_list.lock().expect("Failed to lock file list");
							file_list.push(path);
						});
					}
					Err(e) => {
						eprintln!("Failed to accept client connection: {}", e);
						break;
					}
				}
			}
		});

		self.listener_thread = Some(handle);
	}
}

impl Clone for FileReader {
	fn clone(&self) -> Self {
		Self {
			file_list: Arc::clone(&self.file_list),
			listener_thread: None, // Clones don't get the listener thread
		}
	}
}
