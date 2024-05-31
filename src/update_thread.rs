use crate::{
	command::{match_input, CommandReturn},
	config::Config,
	duration::{display_duration, display_duration_out_of},
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
	path::{Path, PathBuf},
	sync::mpsc::Receiver,
	thread::sleep,
	time::{Duration, Instant},
};

const SLEEP_TIME: u64 = 250;

// Function to update and render changing information in a separate thread
pub fn main(receiver: &Receiver<String>) {
	// load config
	let mut config = Config::load().expect("config should be valid RON");

	// prepend first song if opened with command line args
	let args = std::env::args_os().collect::<Vec<OsString>>();
	if args.len() > 1 {
		config.remaining.insert(0, PathBuf::from(args[1].clone()));
		config.progress = Duration::ZERO;
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
		load_first_song_and_set_name(
			&mut config,
			&sink,
			&mut current_song_name,
			&mut current_song,
		);
	}

	// starting sink values
	let insta_start = config.start_playing_immediately && !config.remaining.is_empty();
	if insta_start || args.len() > 1 {
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
				Ok(CommandReturn::Nothing) => (),
				Ok(CommandReturn::Quit) => break,
				Ok(CommandReturn::QuitNoSave) => return,
				Err(e) => println!("Error: {}", e),
			}

			if config.remaining.is_empty() {
				remaining_songs_ended(&mut config, &sink, &mut current_song_name);
			}

			print_song_info(&current_song_name, &config);
		}

		// update progress time
		if !sink.is_paused() {
			config.progress += last_loop_time.elapsed();
		}

		// go to the next song if the current one is finished
		if sink.empty() && !config.remaining.is_empty() {
			let first = config.remaining[0].clone();
			if current_song == first {
				try_update_song_duration(&mut config, &first);
				config.remaining.remove(0);
			}

			config.progress = Duration::ZERO;
			config.dont_save_at = Duration::ZERO;
			if config.remaining.is_empty() {
				remaining_songs_ended(&mut config, &sink, &mut current_song_name);
			}
			if !config.remaining.is_empty() {
				load_first_song_and_set_name(
					&mut config,
					&sink,
					&mut current_song_name,
					&mut current_song,
				);
			}
			print_song_info(&current_song_name, &config);
		}

		if config.show_song_progress {
			print_song_progress(&config);
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
	// clear the line of the song length to make sure it renders correctly
	execute!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
	// clear the line below the song length, because text can easily glitch to there
	println!();
	execute!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
}

fn print_song_progress(config: &Config) {
	// this intentionally does not clear the line, to avoid flickering
	execute!(stdout(), MoveTo(0, 2)).unwrap();
	if let Some(song_duration) = config.get_current_duration() {
		let s = display_duration_out_of(config.progress, song_duration);
		println!("{}", s);
	} else {
		println!("{}", display_duration(config.progress));
	}
}

// TODO handle errors of the following functions better

fn load_first_song_and_set_name(
	config: &mut Config,
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

pub fn load_first_song(config: &mut Config, sink: &Sink) {
	sink.stop();

	let Some(first) = config.remaining.first().cloned() else {
		println!("Tried playing the first song without a Playlist");
		return;
	};
	let path = {
		if first.is_absolute() {
			first.clone()
		} else {
			config.parent_path.join(&first)
		}
	};
	let file = match File::open(path.clone()) {
		Ok(file) => file,
		Err(_) => {
			println!("Failed to load song: {}", config.remaining[0].display());
			config.remaining.remove(0);
			load_first_song(config, sink);
			return;
		}
	};
	let file = BufReader::new(file);

	#[cfg(feature = "mp4")]
	let decoder = if path
		.file_name()
		.unwrap()
		.to_string_lossy()
		.ends_with(".mp4")
	{
		Decoder::new_mp4(file, rodio::decoder::Mp4Type::Mp4)
			.expect("unable to convert mp4 file to a music file")
	} else {
		Decoder::new(file).expect("unable to convert file to a music file")
	};

	#[cfg(not(feature = "mp4"))]
	let decoder = Decoder::new(file).expect("unable to convert file to a music file");

	let source = decoder.skip_duration(config.progress);
	sink.append(source);
}

fn remaining_songs_ended(config: &mut Config, sink: &Sink, current_song_name: &mut String) {
	if !config.looping_songs.is_empty() {
		config.remaining = config.looping_songs.clone();
		config.progress = Duration::ZERO;
	} else {
		sink.pause();
		*current_song_name = "[No Songs Remaining]".to_owned();
		config.progress = Duration::ZERO;
	}
}

fn try_update_song_duration(config: &mut Config, first: &Path) {
	let d1 = config.progress.as_secs_f32();
	let d2 = config.dont_save_at.as_secs_f32();
	let time_since_skip = (d1 - d2).abs();
	if time_since_skip > 1. {
		if let Some(file) = config.files.get_mut(first) {
			file.duration = Some(config.progress);
		}
	}
}
