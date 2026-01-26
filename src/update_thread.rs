use crate::{
	command::{match_input, run_macro_or, CommandReturn},
	data::{
		context::{Context, ProgramMode},
		files::{get_temp_mp4_filepath, is_mp4_file, make_temp_mp4_copy, FileData},
		playlist::Playlist,
	},
	duration::{display_duration, display_duration_out_of},
	input_thread::INPUT_Y,
	shmem_reader::FileReader,
};
use crossterm::{
	cursor::{MoveTo, RestorePosition, SavePosition},
	execute,
	terminal::{Clear, ClearType},
};
use rodio::{Decoder, Source};
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

// this must be used inside the main loop
macro_rules! handle_command_return {
	($ctx:expr, $csn:expr, $cs:expr, $command_return:expr) => {
		match $command_return {
			Ok(CommandReturn::Nothing) => (),
			Ok(CommandReturn::Quit) => break,
			Ok(CommandReturn::QuitNoSave) => return,
			Ok(CommandReturn::ReloadFirstSong) => load_first_song_and_set_name($ctx, $csn, $cs),
			Err(e) => println!("Error: {}", e),
		}
	};
}

// Function to update and render changing information in a separate thread
pub fn main(receiver: &Receiver<String>, server: Option<FileReader>) {
	let args = env::args_os().collect::<Vec<OsString>>();
	let mut ctx = if let [_, file, ..] = args.as_slice() {
		Context::new_temp(Path::new(file))
	} else {
		Context::new_main()
	}
	.expect("could not load context");

	// load in first song
	let mut last_loop_time = Instant::now();
	let mut current_song_name = String::new();
	let mut current_song = PathBuf::new();

	if ctx.playlist.remaining.is_empty() {
		remaining_songs_ended(&mut ctx, &mut current_song_name);
	}
	if !ctx.playlist.remaining.is_empty() {
		load_first_song_and_set_name(&mut ctx, &mut current_song_name, &mut current_song);
	}

	execute!(stdout(), SavePosition).expect("Failed to save cursor position.");
	print_song_info(&current_song_name, &ctx.playlist);
	execute!(stdout(), RestorePosition).expect("Failed to restore cursor position.");

	// Update and render loop
	loop {
		execute!(stdout(), SavePosition).expect("Failed to save cursor position.");

		// Recieve newly opened files
		if let Some(server) = &server {
			let paths = server.drain_file_list();
			if !paths.is_empty() {
				println!("\n");
				for path in paths.into_iter().filter(|p| p.is_file()) {
					println!(
						"Added Song: {}",
						path.file_name().unwrap().to_string_lossy()
					);
					ctx.playlist.remaining.insert(0, path.clone());
					ctx.files.mappings.insert(path, FileData::default());
				}
				ctx.playlist.progress = Duration::ZERO;
				load_first_song_and_set_name(&mut ctx, &mut current_song_name, &mut current_song);
				print_song_info(&current_song_name, &ctx.playlist);
				ctx.sink.play();
			}
		}

		// Receive user input (if any)
		if let Ok(input) = receiver.try_recv() {
			execute!(
				stdout(),
				MoveTo(0, INPUT_Y + 2),
				Clear(ClearType::FromCursorDown)
			)
			.expect("Failed to execute cursor movement and clear.");

			let was_not_empty = !ctx.playlist.remaining.is_empty();

			handle_command_return! {
				&mut ctx, &mut current_song_name, &mut current_song,
				match_input(&input, &mut ctx)
			};

			if ctx.playlist.remaining.is_empty() {
				ctx.sink.pause();
			}

			if was_not_empty && ctx.playlist.remaining.is_empty() {
				remaining_songs_ended(&mut ctx, &mut current_song_name);
				handle_command_return! {
					&mut ctx, &mut current_song_name, &mut current_song,
					run_macro_or(&mut ctx, "@list_end", &[], "")
				};
			}

			print_song_info(&current_song_name, &ctx.playlist);
		}

		// update progress time
		if !ctx.sink.is_paused() {
			ctx.playlist.progress += last_loop_time.elapsed();
		}

		// go to the next song if the current one is finished
		if ctx.sink.empty() && !ctx.playlist.remaining.is_empty() {
			let first = ctx.playlist.remaining[0].clone();
			try_update_song_duration(&mut ctx, &first);
			ctx.playlist.remaining.remove(0);
			handle_command_return! {
				&mut ctx, &mut current_song_name, &mut current_song,
				run_macro_or(&mut ctx, "@song_end", &[], "")
			};

			ctx.playlist.progress = Duration::ZERO;
			if ctx.playlist.remaining.is_empty() {
				remaining_songs_ended(&mut ctx, &mut current_song_name);
				handle_command_return! {
					&mut ctx, &mut current_song_name, &mut current_song,
					run_macro_or(&mut ctx, "@list_end", &[], "")
				};
			}
			if !ctx.playlist.remaining.is_empty() {
				load_first_song_and_set_name(&mut ctx, &mut current_song_name, &mut current_song);
				handle_command_return! {
					&mut ctx, &mut current_song_name, &mut current_song,
					run_macro_or(&mut ctx, "@song_start", &[], "")
				};
			}
			print_song_info(&current_song_name, &ctx.playlist);
		}

		if ctx.config.show_song_progress {
			print_song_progress(&ctx);
		}

		// move the cursor back to allow for user input to not be glitchy
		execute!(stdout(), RestorePosition).expect("Failed to restore cursor position.");

		last_loop_time = Instant::now();

		// Adjust the sleep duration based on how often you want to update the display
		sleep(Duration::from_millis(SLEEP_TIME));
	}

	if ctx.program_mode.can_save() {
		ctx.config.save().expect("Failed to save config.");
		ctx.files.save().expect("Failed to save files.");
		ctx.playlist
			.save(&ctx.config.current_playlist)
			.expect("Failed to save playlist.");
	}
}

fn print_song_info(current_song_name: &String, playlist: &Playlist) {
	execute!(stdout(), MoveTo(0, 0), Clear(ClearType::CurrentLine))
		.expect("Failed moving cursor to the top");
	println!("Song: {current_song_name}");
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
		println!("{s}");
	} else {
		println!("{}", display_duration(ctx.playlist.progress));
	}
}

// TODO handle errors of the following functions better

fn load_first_song_and_set_name(ctx: &mut Context, song_name: &mut String, song: &mut PathBuf) {
	load_first_song(ctx);

	let Some(first) = ctx.playlist.remaining.first().cloned() else {
		// cant call @list_end event here, since we are not in the main loop
		remaining_songs_ended(ctx, song_name);
		return;
	};

	*song = first.clone();

	*song_name = first
		.file_name()
		.expect("Failed to get file name from the path.")
		.to_string_lossy()
		.to_string();
}

fn load_first_song(ctx: &mut Context) {
	ctx.sink.stop();

	let (file, first) = loop {
		let Some(first) = ctx.playlist.remaining.first().cloned() else {
			return;
		};
		// you may have relative paths in temp mode that are not relative to
		// ctx.files.root, because this program can be started with relative
		// command line arguments to specify a music file.
		let mut path = if first.is_absolute() || ctx.program_mode == ProgramMode::Temp {
			first.clone()
		} else {
			ctx.files.root.join(&first)
		};
		if is_mp4_file(&path.to_string_lossy()) {
			match make_temp_mp4_copy(&path).and_then(|()| get_temp_mp4_filepath()) {
				Ok(p) => path = p,
				Err(_) => {
					println!("Failed to convert song to mp3: {}", first.display());
					ctx.playlist.remaining.remove(0);
					continue;
				}
			}
		}
		match File::open(path) {
			Ok(file) => break (file, first),
			Err(_) => {
				println!("Failed to load song: {}", first.display());
				ctx.playlist.remaining.remove(0);
				continue;
			}
		};
	};
	let file = BufReader::new(file);

	let mut decoder = Decoder::new(file).expect("unable to convert file to a music file");

	// update the cached duration to be accurate if the decoder type supports it
	if let Some(total) = decoder.total_duration() {
		if let Some(file) = ctx.files.get_mut(&first) {
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

fn remaining_songs_ended(ctx: &mut Context, song_name: &mut String) {
	ctx.sink.pause();
	*song_name = "[No Songs Remaining]".to_owned();
	ctx.playlist.progress = Duration::ZERO;
}

/// sets the cached duration of `song` to the progress of the current song
fn try_update_song_duration(ctx: &mut Context, song: &Path) {
	if let Some(file) = ctx.files.get_mut(song) {
		file.duration = Some(ctx.playlist.progress);
	}
}
