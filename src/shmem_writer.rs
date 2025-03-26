use std::{fs::OpenOptions, path::PathBuf};

use memmap2::MmapMut;

use crate::shmem_reader::{get_shm_file_path, SHM_SIZE};

pub struct FileWriter;

impl FileWriter {
	pub fn new() -> Self {
		Self
	}

	pub fn send_to_existing_instance(&self, file: PathBuf) -> bool {
		// Get the correct shared memory file path for the platform
		let shm_file = get_shm_file_path();

		// Check if the shared memory file exists (indicating that the server is running)
		if !shm_file.exists() {
			eprintln!("No active server detected (shared memory file does not exist).");
			return false;
		}

		// Attempt to open the shared memory file
		let file_handle = match OpenOptions::new().read(true).write(true).open(&shm_file) {
			Ok(f) => f,
			Err(e) => {
				eprintln!("Failed to open shared memory file: {}", e);
				return false;
			}
		};

		// SAFETY: We assume that the file exists and has the correct size
		let mut mmap = unsafe {
			match MmapMut::map_mut(&file_handle) {
				Ok(m) => m,
				Err(e) => {
					eprintln!("Failed to map shared memory: {}", e);
					return false;
				}
			}
		};

		let message = file.to_string_lossy().to_string() + "\n";
		let mut write_pos = 0;

		// Find the first available position in the shared memory
		while write_pos + message.len() < SHM_SIZE {
			if mmap[write_pos] == 0 {
				break;
			}
			write_pos += 1;
		}

		// Ensure there is enough space to write the message
		if write_pos + message.len() >= SHM_SIZE {
			eprintln!("Shared memory full!");
			return false;
		}

		// Copy the message into shared memory
		mmap[write_pos..(write_pos + message.len())].copy_from_slice(message.as_bytes());

		// Flush changes to ensure they are visible
		if mmap.flush().is_err() {
			eprintln!("Failed to flush shared memory!");
			return false;
		}

		true
	}
}
