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
	shutdown_flag: Arc<(Mutex<bool>, std::sync::Condvar)>,
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
			.unwrap_or_else(|_| {
				panic!(
					"Failed to create or open shared memory file at {:?}",
					shm_file
				)
			});

		// Ensure the file has the correct size
		file_handle
			.set_len(SHM_SIZE as u64)
			.unwrap_or_else(|_| panic!("Failed to set shared memory file size at {:?}", shm_file));

		// SAFETY: The file exists and has the correct size, so we can safely map it
		let mut mmap = unsafe {
			MmapMut::map_mut(&file_handle)
				.unwrap_or_else(|_| panic!("Failed to map shared memory file at {:?}", shm_file))
		};
		mmap.fill(0); // Clear shared memory

		Self {
			file_list: Arc::new(Mutex::new(Vec::new())),
			shutdown_flag: Arc::new((Mutex::new(false), std::sync::Condvar::new())),
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
				.unwrap_or_else(|_| panic!("Failed to open shared memory file at {:?}", shm_file));

			// SAFETY: We assume the file is valid and mapped correctly
			let mut mmap = unsafe {
				MmapMut::map_mut(&file_handle).unwrap_or_else(|_| {
					panic!("Failed to map shared memory file at {:?}", shm_file)
				})
			};

			loop {
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
							.expect("Failed to acquire file list lock in the server thread");
						list.extend(new_files);
						println!("Updated file list: {:?}", *list);
					}
					mmap.fill(0); // Clear shared memory after processing
				}

				let (lock, cvar) = &*shutdown_flag_clone;

				let mut started = lock
					.lock()
					.expect("Failed to acquire shutdown lock in the server thread");
				let result = cvar
					.wait_timeout(started, Duration::from_millis(500))
					.expect("Failed to wait on the condition variable in the server thread");
				started = result.0;

				if *started {
					return;
				}
			}
		});
	}

	pub fn close_server(&self) {
		// Signal the server thread to stop
		{
			let (lock, cvar) = &*self.shutdown_flag;
			let mut started = lock
				.lock()
				.expect("Failed to acquire shutdown lock in close_server");
			*started = true;
			cvar.notify_one(); // Wake the sleeping thread
		}

		// Get the correct shared memory file path for the platform
		let shm_file = get_shm_file_path();

		// Try to delete the shared memory file
		if let Err(e) = std::fs::remove_file(&shm_file) {
			eprintln!(
				"Failed to delete shared memory file at {:?}: {}",
				shm_file, e
			);
		} else {
			println!("Shared memory file deleted successfully at {:?}", shm_file);
		}
	}

	pub fn drain_file_list(&self) -> Vec<PathBuf> {
		// Lock and drain the file list safely
		self.file_list
			.lock()
			.expect("Failed to acquire file list lock in drain_file_list")
			.drain(..)
			.collect()
	}
}
