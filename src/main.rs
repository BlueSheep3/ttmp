#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::infinite_loop)]
#![warn(clippy::use_self)]

mod command;
mod data;
mod duration;
mod input_thread;
mod macros;
mod pause_thread;
mod serializer;
mod shmem_reader;
mod shmem_writer;
mod update_thread;

use std::{env, path::PathBuf, process::exit, sync::mpsc::channel, thread};

use shmem_reader::FileReader;
use shmem_writer::FileWriter;
use std::sync::{Arc, Mutex};

fn main() {
	let file = env::args_os().nth(1).map(PathBuf::from);
	let client = FileWriter::new();

	// If another instance is running, send the file and exit
	if let Some(file) = file {
		if client.send_to_existing_instance(file) {
			exit(0);
		}
	}

	// Wrap the server in an Arc and Mutex for shared ownership
	let server = Arc::new(Mutex::new(FileReader::new()));
	server
		.lock()
		.expect("current thread is already holding server")
		.start_server();

	// Create channels for communication between threads
	let (sender, receiver) = channel();
	let sender_clone = sender.clone();

	// Spawn threads for user input and updating/rendering
	let _input_thread = thread::spawn(move || input_thread::main(&sender));
	let update_thread = thread::spawn({
		let server_clone_2 = Arc::clone(&server);
		move || update_thread::main(&receiver, &server_clone_2)
	});
	let _pause_thread = thread::spawn(move || pause_thread::main(&sender_clone));

	// wait for update thread to finish before exiting
	// don't wait for input thread, because it only handles input, not quitting
	if let Err(e) = update_thread.join() {
		println!("Failed to join update thread: {:?}", e);
		readln!();
	}

	// Lock the server before stopping
	server
		.lock()
		.expect("current thread is already holding server")
		.close_server();
}
