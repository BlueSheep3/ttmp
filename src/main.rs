mod command;
mod config;
mod input_thread;
mod macros;
mod serializer;
mod update_thread;

use std::{sync::mpsc::channel, thread};

type DynErr = Box<dyn std::error::Error>;

fn main() {
	// Create channels for communication between threads
	let (sender, receiver) = channel();

	// Spawn threads for user input and updating/rendering
	let _input_thread = thread::spawn(move || input_thread::main(&sender));
	let update_thread = thread::spawn(move || update_thread::main(&receiver));

	// wait for update thread to finish before exiting
	// dont wait for input thread, because it only handles input, not quitting
	if let Err(e) = update_thread.join() {
		println!("Failed to join update thread: {:?}", e);
		readln!();
	}
}
