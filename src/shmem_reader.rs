use std::{
	env,
	fs::OpenOptions,
	path::PathBuf,
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};

use memmap2::MmapMut;

pub const SHM_SIZE: usize = 4096; // Shared memory size

pub fn get_shm_file_path() -> PathBuf {
	const FILE_NAME: &str = "ipc_shmem_music_player_dfjvndiergnasinisdjvnf";
	if cfg!(target_os = "windows") {
		// Use a path that works on Windows (for example, in the current user's temp directory)
		let temp_dir = env::temp_dir();
		temp_dir.join(FILE_NAME)
	} else {
		// Use a Unix-like path on non-Windows platforms
		PathBuf::from(format!("/tmp/{}", FILE_NAME))
	}
}

pub struct FileReader {
	file_list: Arc<Mutex<Vec<PathBuf>>>,
	shutdown_flag: Arc<Mutex<bool>>, // Add shutdown flag
}

impl FileReader {
	pub fn new() -> Self {
		// Get the correct shared memory file path for the platform
		let shm_file = get_shm_file_path();

		// Create or truncate the shared memory file
		let file_handle = OpenOptions::new()
			.create(true)
			.read(true)
			.write(true)
			.truncate(true)
			.open(&shm_file)
			.expect("Failed to create or open shared memory file");

		// Ensure the file has the correct size
		file_handle
			.set_len(SHM_SIZE as u64)
			.expect("Failed to set shared memory file size");

		// SAFETY: The file exists and has the correct size, so we can safely map it
		let mut mmap =
			unsafe { MmapMut::map_mut(&file_handle).expect("Failed to map shared memory file") };
		mmap.fill(0); // Clear shared memory

		Self {
			file_list: Arc::new(Mutex::new(Vec::new())),
			shutdown_flag: Arc::new(Mutex::new(false)), // Initialize shutdown flag
		}
	}

	pub fn start_server(&self) {
		let file_list_clone = Arc::clone(&self.file_list);
		let shutdown_flag_clone = Arc::clone(&self.shutdown_flag);

		thread::spawn(move || {
			// Get the correct shared memory file path for the platform
			let shm_file = get_shm_file_path();

			// Attempt to open the shared memory file
			let file_handle = OpenOptions::new()
				.read(true)
				.write(true)
				.open(&shm_file)
				.expect("Failed to open shared memory file in server");

			// SAFETY: We assume the file is valid and mapped correctly
			let mut mmap = unsafe {
				MmapMut::map_mut(&file_handle).expect("Failed to map shared memory in server")
			};

			loop {
				// Check if we should shutdown the server thread
				if *shutdown_flag_clone
					.lock()
					.expect("Failed to lock shutdown flag")
				{
					break; // Exit the loop and stop the thread
				}

				// Read the contents of shared memory
				let buffer: String = mmap
					.iter()
					.filter(|&&c| c != 0)
					.map(|&c| c as char)
					.collect();

				// If there is new data, process it
				if !buffer.is_empty() {
					let new_files: Vec<PathBuf> = buffer.lines().map(PathBuf::from).collect();
					{
						let mut list = file_list_clone
							.lock()
							.expect("Failed to acquire file list lock");
						list.extend(new_files);
						println!("Updated file list: {:?}", *list);
					}
					mmap.fill(0); // Clear shared memory after processing
				}
				thread::sleep(Duration::from_secs(1));
			}
		});
	}

	pub fn close_server(&self) {
		// Signal the server thread to stop
		{
			let mut shutdown_flag = self
				.shutdown_flag
				.lock()
				.expect("Failed to lock shutdown flag");
			*shutdown_flag = true; // Set the shutdown flag to true
		}

		// Wait for the server thread to stop (you might want a better mechanism for waiting in a real app)
		thread::sleep(Duration::from_secs(1)); // Just a simple sleep, you might use a more efficient method like `join` if needed

		// Get the correct shared memory file path for the platform
		let shm_file = get_shm_file_path();

		// Try to delete the shared memory file
		if let Err(e) = std::fs::remove_file(&shm_file) {
			eprintln!("Failed to delete shared memory file: {}", e);
		} else {
			println!("Shared memory file deleted successfully.");
		}
	}

	pub fn drain_file_list(&self) -> Vec<PathBuf> {
		// Lock and drain the file list safely
		self.file_list
			.lock()
			.expect("Failed to acquire file list lock")
			.drain(..)
			.collect()
	}
}
