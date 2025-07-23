#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![warn(clippy::undocumented_unsafe_blocks)]
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

use crossterm::{
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use shmem_reader::FileReader;
use std::{env, io::stdout, path::PathBuf, sync::mpsc::channel, thread};

fn main() {
	let pipe_name = "//./pipe/ipc_music_player_xmyuiwqcoecmztrciqenasjkf";
	let file = env::args_os().nth(1).map(PathBuf::from);

	let mut server = None;
	if let Some(file) = file {
		// if the path is relative, this was most likely manually started
		// in a shell, in which case we want the process to be isolated
		if file.is_absolute() {
			let file = file.canonicalize().unwrap();

			// if another instance is running, send the file and exit
			if shmem_writer::try_send_to_pipe(pipe_name, file) {
				return;
			}

			let reader = FileReader::default();
			reader.start_receiving(pipe_name);
			server = Some(reader);
		}
	}

	execute!(stdout(), EnterAlternateScreen).expect("Failed to enter alternate screen.");

	// Create channels for communication between threads
	let (sender, receiver) = channel();
	let sender_clone = sender.clone();

	// Spawn threads for user input and updating/rendering
	let _input_thread = thread::spawn(move || input_thread::main(&sender));
	let update_thread = thread::spawn(move || update_thread::main(&receiver, server));
	let _pause_thread = thread::spawn(move || pause_thread::main(&sender_clone));

	// wait for update thread to finish before exiting
	// don't wait for input thread, because it only handles input, not quitting
	if let Err(e) = update_thread.join() {
		println!("Failed to join update thread: {e:?}");
		readln!();
	}

	execute!(stdout(), LeaveAlternateScreen).expect("Failed to leave alternate screen.");
}
