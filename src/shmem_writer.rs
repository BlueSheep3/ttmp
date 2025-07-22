use std::{
	fs::OpenOptions,
	io::{BufWriter, Write},
	path::PathBuf,
};

pub struct FileWriter;

impl FileWriter {
	pub fn new() -> Self {
		Self {}
	}

	pub fn send_to_existing_instance(&self, pipe_name: &str, file_path: PathBuf) -> bool {
		let file_str = match file_path.to_str() {
			Some(s) => s,
			None => {
				eprintln!("Invalid file path (non-UTF8 characters)");
				return false;
			}
		};

		let file = match OpenOptions::new().write(true).open(pipe_name) {
			Ok(f) => f,
			Err(e) => {
				eprintln!("Failed to open pipe '{pipe_name}': {e}");
				return false;
			}
		};

		let mut writer = BufWriter::new(file);
		match writer.write_all(file_str.as_bytes()) {
			Ok(_) => {
				if let Err(e) = writer.flush() {
					eprintln!("Failed to flush pipe: {e}");
					return false;
				}
				println!("Sent path: {file_str}");
				true
			}
			Err(e) => {
				eprintln!("Failed to write to pipe: {e}");
				false
			}
		}
	}
}
