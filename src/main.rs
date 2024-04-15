mod command;
mod config;
mod input_thread;
mod macros;
mod pause_thread;
mod serializer;
mod update_thread;

use std::{sync::mpsc::channel, thread};

type DynErr = Box<dyn std::error::Error>;

fn main() {
	// Create channels for communication between threads
	let (sender, receiver) = channel();

	let sender_clone = sender.clone();

	// Spawn threads for user input and updating/rendering
	let _input_thread = thread::spawn(move || input_thread::main(&sender));
	let update_thread = thread::spawn(move || update_thread::main(&receiver));
	let _pause_thread = thread::spawn(move || pause_thread::main(&sender_clone));

	// wait for update thread to finish before exiting
	// dont wait for input thread, because it only handles input, not quitting
	if let Err(e) = update_thread.join() {
		println!("Failed to join update thread: {:?}", e);
		readln!();
	}
}
