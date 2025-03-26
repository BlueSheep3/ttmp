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

		let binding = file.to_string_lossy().to_string();
		let message = binding.as_bytes();
		let length = message.len();
		if message.is_empty() {
			panic!("Tried to send message with no data")
		}
		let mut write_pos = 0;

		// Find the first available position in the shared memory
		while write_pos + length < SHM_SIZE {
			let size = read_length(&mmap, &mut write_pos);
			if size == 0 {
				write_pos -= 1;
				break;
			}
			write_pos += size as usize;
		}

		// Ensure there is enough space to write the message
		if write_pos + length >= SHM_SIZE {
			eprintln!("Shared memory full!");
			return false;
		}

		// Copy the message into shared memory
		write_length(&mut mmap, &mut write_pos, length);
		mmap[write_pos..(write_pos + length)].copy_from_slice(message);

		// Flush changes to ensure they are visible
		if mmap.flush().is_err() {
			eprintln!("Failed to flush shared memory!");
			return false;
		}

		true
	}
}

pub fn read_length(mmap: &MmapMut, pos: &mut usize) -> usize {
	let mut size = mmap[*pos] as usize;
	if size >= 0x100 {
		size -= 0x100;
		*pos += 1;
		let next_bits = mmap[*pos] as usize;
		size = (size << 8) | next_bits;
	}
	*pos += 1;
	size
}

pub fn write_length(mmap: &mut MmapMut, pos: &mut usize, length: usize) {
	if length >= 0x100 {
		mmap[*pos] = ((length >> 8) & 0xFF) as u8;
		*pos += 1;
		mmap[*pos] = (length & 0xFF) as u8;
	} else {
		mmap[*pos] = length as u8;
	}
	*pos += 1;
}
