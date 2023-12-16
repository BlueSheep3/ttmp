use crate::{
	command::{match_input, CommandReturn},
	config::{load as load_config, Config},
	input_thread::INPUT_Y,
};
use crossterm::{
	cursor::{MoveTo, RestorePosition, SavePosition},
	execute,
	terminal::{Clear, ClearType},
};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
	ffi::OsString,
	fs::File,
	io::{stdout, BufReader},
	path::PathBuf,
	sync::mpsc::Receiver,
	thread::sleep,
	time::{Duration, Instant},
};

const SLEEP_TIME: u64 = 250;

// Function to update and render changing information in a separate thread
pub fn main(receiver: &Receiver<String>) {
	// load config
	let mut config = load_config().expect("config should be valid RON");

	// prepend first song if opened with command line args
	let args = std::env::args_os().collect::<Vec<OsString>>();
	if args.len() > 1 {
		config.remaining.insert(0, PathBuf::from(args[1].clone()));
		config.current_progress = Duration::ZERO;
	}

	// setup sink
	let (_stream, stream_handle) =
		OutputStream::try_default().expect("Failed to create audio stream.");
	let sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink.");

	// load in first song
	let mut last_loop_time = Instant::now();
	let mut current_song_name = String::new();
	let mut current_song = PathBuf::new();

	if config.remaining.is_empty() {
		remaining_songs_ended(&mut config, &sink, &mut current_song_name);
	}
	if !config.remaining.is_empty() {
		load_first_song_and_set_name(&config, &sink, &mut current_song_name, &mut current_song);
	}

	// starting sink values
	if config.start_playing_immediately || args.len() > 1 {
		sink.play();
	} else {
		sink.pause();
	}
	sink.set_speed(config.speed);
	sink.set_volume(config.volume);

	execute!(stdout(), SavePosition).expect("Failed to save cursor position.");
	print_song_info(&current_song_name, &config);
	execute!(stdout(), RestorePosition).expect("Failed to restore cursor position.");

	// Update and render loop
	loop {
		execute!(stdout(), SavePosition).expect("Failed to save cursor position.");

		// Receive user input (if any)
		if let Ok(input) = receiver.try_recv() {
			execute!(
				stdout(),
				MoveTo(0, INPUT_Y + 2),
				Clear(ClearType::FromCursorDown)
			)
			.expect("Failed to execute cursor movement and clear.");

			let state = match_input(&input, &sink, &mut config);
			match state {
				CommandReturn::Nothing => (),
				CommandReturn::Quit => break,
				CommandReturn::QuitNoSave => return,
			}

			if config.remaining.is_empty() {
				remaining_songs_ended(&mut config, &sink, &mut current_song_name);
			}

			print_song_info(&current_song_name, &config);
		}

		// update progress time
		if !sink.is_paused() {
			config.current_progress += last_loop_time.elapsed();
		}

		// go to the next song if the current one is finished
		if sink.empty() && !config.remaining.is_empty() {
			if config.move_file_soon != PathBuf::new() {
				let move_file = config.move_file_soon.clone();
				crate::command::misc::move_file(&mut config, move_file);
				config.move_file_soon = PathBuf::new();
			}

			if current_song == config.remaining[0] {
				config.remaining.remove(0);
			}

			config.current_progress = Duration::ZERO;
			if config.remaining.is_empty() {
				remaining_songs_ended(&mut config, &sink, &mut current_song_name);
			}
			if !config.remaining.is_empty() {
				load_first_song_and_set_name(
					&config,
					&sink,
					&mut current_song_name,
					&mut current_song,
				);
			}
			print_song_info(&current_song_name, &config);
		}

		// move the cursor back to allow for user input to not be glitchy
		execute!(stdout(), RestorePosition).expect("Failed to restore cursor position.");

		last_loop_time = Instant::now();

		// Adjust the sleep duration based on how often you want to update the display
		sleep(Duration::from_millis(SLEEP_TIME));
	}

	config.save().expect("Failed to save config.");
}

fn print_song_info(current_song_name: &String, config: &Config) {
	execute!(stdout(), MoveTo(0, 0), Clear(ClearType::CurrentLine)).unwrap();
	println!("Music: {}", current_song_name);
	execute!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
	println!("Songs Remaining: {}", config.remaining.len());
	execute!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
}

fn load_first_song_and_set_name(
	config: &Config,
	sink: &Sink,
	song_name: &mut String,
	song: &mut PathBuf,
) {
	load_first_song(config, sink);

	let Some(first) = config.remaining.first().cloned() else {
		panic!("Tried playing the first song without a Playlist");
	};

	*song = first.clone();

	*song_name = first
		.file_name()
		.expect("Failed to get file name from the path.")
		.to_string_lossy()
		.to_string();
}

pub fn load_first_song(config: &Config, sink: &Sink) {
	let Some(first) = config.remaining.first().cloned() else {
		panic!("Tried playing the first song without a Playlist");
	};
	let path = {
		if first.is_absolute() {
			first.clone()
		} else {
			config.parent_path.join(&first)
		}
	};
	let file = File::open(path).expect("unable to open file");
	let file = BufReader::new(file);
	// mp4 crashes in let source = ...
	let source = Decoder::new(file).expect("unable to convert file to a music file");
	let source = source.skip_duration(config.current_progress);
	sink.append(source);
}

fn remaining_songs_ended(config: &mut Config, sink: &Sink, current_song_name: &mut String) {
	if !config.looping_songs.is_empty() {
		config.remaining = config.looping_songs.clone();
		config.current_progress = Duration::ZERO;
	} else {
		sink.pause();
		*current_song_name = "[No Songs Remaining]".to_owned();
	}
}
