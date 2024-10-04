use crate::{
	command::{match_input, CommandReturn},
	data::{config::Config, context::Context, playlist::Playlist},
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
	env,
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
	let args = env::args_os().collect::<Vec<OsString>>();
	let ctx = if let [_, file, ..] = args.as_slice() {
		Context::new_temp(Path::new(file))
	} else {
		Context::new_main()
	}
	.expect("could not load context");

	let mut ctx @ Context {
		program_mode,
		config,
		playlist,
		sink,
		..
	} = ctx;

	// load in first song
	let mut last_loop_time = Instant::now();
	let mut current_song_name = String::new();
	let mut current_song = PathBuf::new();

	if playlist.remaining.is_empty() {
		remaining_songs_ended(&mut config, &sink, &mut current_song_name);
	}
	if !playlist.remaining.is_empty() {
		load_first_song_and_set_name(
			&mut config,
			&sink,
			&mut current_song_name,
			&mut current_song,
		);
	}

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

fn print_song_info(current_song_name: &String, playlist: &Playlist) {
	execute!(stdout(), MoveTo(0, 0), Clear(ClearType::CurrentLine))
		.expect("Failed moving cursor to the top");
	println!("Music: {}", current_song_name);
	execute!(stdout(), Clear(ClearType::CurrentLine)).expect("Failed clearing line");
	println!("Songs Remaining: {}", playlist.remaining.len());
	// clear the line of the song length to make sure it renders correctly
	execute!(stdout(), Clear(ClearType::CurrentLine)).expect("Failed clearing line");
	// clear the line below the song length, because text can easily glitch to there
	println!();
	execute!(stdout(), Clear(ClearType::CurrentLine)).expect("Failed clearing line");
}

fn print_song_progress(ctx: &Context) {
	// this intentionally does not clear the line, to avoid flickering
	execute!(stdout(), MoveTo(0, 2)).expect("Failed moving cursor");
	if let Some(song_duration) = ctx.get_current_duration() {
		let s = display_duration_out_of(ctx.playlist.progress, song_duration);
		println!("{}", s);
	} else {
		println!("{}", display_duration(ctx.playlist.progress));
	}
}

// TODO handle errors of the following functions better

fn load_first_song_and_set_name(ctx: &mut Context, song_name: &mut String, song: &mut PathBuf) {
	load_first_song(ctx);

	let Some(first) = ctx.playlist.remaining.first().cloned() else {
		panic!("Tried playing the first song without a Playlist");
	};

	*song = first.clone();

	*song_name = first
		.file_name()
		.expect("Failed to get file name from the path.")
		.to_string_lossy()
		.to_string();
}

pub fn load_first_song(ctx: &mut Context) {
	ctx.sink.stop();

	let Some(first) = ctx.playlist.remaining.first().cloned() else {
		println!("Tried playing the first song with an empty Playlist");
		return;
	};
	let path = {
		if first.is_absolute() {
			first.clone()
		} else {
			ctx.config.path.join(&first)
		}
	};
	let file = match File::open(path.clone()) {
		Ok(file) => file,
		Err(_) => {
			println!("Failed to load song: {}", first.display());
			ctx.playlist.remaining.remove(0);
			load_first_song(ctx);
			return;
		}
	};
	let file = BufReader::new(file);

	#[cfg(feature = "mp4")]
	let decoder = if path
		.file_name()
		.expect("unable to get file name of current song")
		.to_string_lossy()
		.ends_with(".mp4")
	{
		Decoder::new_mp4(file, rodio::decoder::Mp4Type::Mp4)
			.expect("unable to convert mp4 file to a music file")
	} else {
		Decoder::new(file).expect("unable to convert file to a music file")
	};

	#[cfg(not(feature = "mp4"))]
	let mut decoder = Decoder::new(file).expect("unable to convert file to a music file");

	// update the cached duration to be accurate if the decoder type supports it
	if let Some(total) = decoder.total_duration() {
		if let Some(file) = ctx.config.files.get_mut(&first) {
			file.duration = Some(total);
		}
	}

	// `try_seek` is a faster alternative to `skip_duration`,
	// but isn't supported for all file formats
	if decoder.try_seek(ctx.playlist.progress).is_ok() {
		ctx.sink.append(decoder);
	} else {
		// logging the error value of `try_seek` would be nice here,
		// but in many cases this would print text over other "ui" elements.
		let source = decoder.skip_duration(ctx.playlist.progress);
		ctx.sink.append(source);
	}
}

fn remaining_songs_ended(ctx: &mut Context, current_song_name: &mut String) {
	ctx.sink.pause();
	*current_song_name = "[No Songs Remaining]".to_owned();
	ctx.playlist.progress = Duration::ZERO;
}

/// sets the cached duration of `song` to the progress of the current song
fn try_update_song_duration(ctx: &mut Context, song: &Path) {
	let d1 = ctx.playlist.progress.as_secs_f32();
	let d2 = ctx.playlist.dont_save_at.as_secs_f32();
	let time_since_skip = (d1 - d2).abs();
	if time_since_skip > 1. {
		if let Some(file) = ctx.config.files.get_mut(song) {
			file.duration = Some(ctx.playlist.progress);
		}
	}
}
