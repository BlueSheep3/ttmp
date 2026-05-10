// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use std::{
	io::Read,
	path::PathBuf,
	sync::{Arc, Mutex},
	thread,
};

#[derive(Default)]
pub struct FileReader {
	file_list: Arc<Mutex<Vec<PathBuf>>>,
}

impl FileReader {
	pub fn drain_file_list(&self) -> Vec<PathBuf> {
		let mut file_list = self.file_list.lock().expect("Failed to lock file list");
		file_list.drain(..).collect()
	}

	pub fn start_receiving(&self, pipe_name: &str) {
		let pipe_name = pipe_name.to_owned();
		let file_list = Arc::clone(&self.file_list);

		#[cfg(target_os = "windows")]
		thread::spawn(move || {
			use interprocess::os::windows::named_pipe::{
				PipeListener, PipeListenerOptions, PipeMode,
				pipe_mode::{Bytes, Messages},
			};
			use std::io::BufReader;

			let listener: PipeListener<Bytes, Messages> = PipeListenerOptions::new()
				.path(&*pipe_name)
				.mode(PipeMode::Messages)
				.create()
				.expect("Failed to create named pipe");
			let mut buffer = String::new();

			loop {
				match listener.accept() {
					Ok(connection) => {
						let mut reader = BufReader::new(connection);

						reader
							.read_to_string(&mut buffer)
							.expect("Failed to read message");

						let path = PathBuf::from(buffer.trim());
						let mut file_list = file_list.lock().expect("Failed to lock file list");
						file_list.push(path);
						buffer.clear();
					}
					Err(e) => {
						eprintln!("Failed to accept client connection: {e}");
						break;
					}
				}
			}
		});

		#[cfg(unix)]
		thread::spawn(move || {
			use std::{
				ffi::OsStr,
				os::unix::{ffi::OsStrExt, net::UnixListener},
			};

			_ = std::fs::remove_file(&pipe_name); // ignore error if file doesnt exist
			let listener = UnixListener::bind(&pipe_name).expect("failed to create unix socket");
			let mut buffer = Vec::new();

			loop {
				match listener.accept() {
					Ok((mut socket, _)) => {
						socket
							.read_to_end(&mut buffer)
							.expect("failed to read message");
						let mut file_list = file_list.lock().expect("failed to lock file list");
						file_list.push(OsStr::from_bytes(&buffer).into());
						buffer.clear();
					}
					Err(e) => {
						eprintln!("Failed to accept client connection: {e}");
						break;
					}
				}
			}
		});
	}
}
