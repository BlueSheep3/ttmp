use crate::{
	Message, Model,
	command::{self, CommandReturn, match_input, run_macro_or},
	data::{
		context::{Context, ProgramMode},
		files::FileData,
	},
};
use rodio::{Decoder, Source};
use std::{
	error::Error,
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
	time::{Duration, Instant},
};

// TEMP temporary hacky solution to handle the CommandReturn values
#[derive(Default)]
struct UpdateTemp {
	quit: bool,
	quit_no_save: bool,
	reload_first_song: bool,
}

pub fn update(
	mut model: Model,
	message: Message,
) -> Result<(Model, Option<Message>), Box<dyn Error>> {
	let mut update_temp = UpdateTemp::default();

	receive_files_over_ipc(&mut model);

	match message {
		Message::DoUpdateAgain => (),
		Message::Quit { .. } => (), // this gets handled in the main loop
		Message::Error => Err("manually triggered error")?,
		Message::Panic => panic!("manually triggered panic"),
		Message::TypedChar(c) => model.currently_typing.push(c),
		Message::Backspace => drop(model.currently_typing.pop()),
		Message::Enter => {
			let was_not_empty = !model.ctx.playlist.remaining.is_empty();

			handle_command_return(
				match_input(&model.currently_typing, &mut model.ctx),
				&mut model.command_output,
				&mut update_temp,
			);

			if model.ctx.playlist.remaining.is_empty() {
				model.ctx.sink.pause();
			}

			if was_not_empty && model.ctx.playlist.remaining.is_empty() {
				remaining_songs_ended(&mut model.ctx, &mut model.current_song_name);
				handle_command_return(
					run_macro_or(&mut model.ctx, "@list_end", &[], ""),
					&mut model.command_output,
					&mut update_temp,
				);
			}

			model.currently_typing.clear();
		}
	}

	if let Ok(()) = model.pause_receiver.try_recv() {
		command::toggle_playing(&mut model.ctx);
	}

	if !model.ctx.sink.is_paused() {
		model.ctx.playlist.progress += model.last_update_time.elapsed();
	}
	maybe_goto_next_song(&mut model, &mut update_temp);
	model.last_update_time = Instant::now();

	let msg = match () {
		() if update_temp.quit => Some(Message::Quit { save: true }),
		() if update_temp.quit_no_save => Some(Message::Quit { save: false }),
		() if update_temp.reload_first_song => {
			load_first_song_and_set_name(
				&mut model.ctx,
				&mut model.current_song_name,
				&mut model.current_song,
			);
			Some(Message::DoUpdateAgain)
		}
		() => None,
	};
	Ok((model, msg))
}

fn receive_files_over_ipc(model: &mut Model) {
	let Some(server) = &model.ipc_server else {
		return;
	};
	let paths = server.drain_file_list();
	if paths.is_empty() {
		return;
	}
	model.command_output.push('\n');
	for path in paths.into_iter().filter(|p| p.is_file()) {
		model.command_output.push_str(&format!(
			"Added Song: {}",
			path.file_name().unwrap().to_string_lossy()
		));
		model.ctx.playlist.remaining.insert(0, path.clone());
		model.ctx.files.mappings.insert(path, FileData::default());
	}
	model.ctx.playlist.progress = Duration::ZERO;
	load_first_song_and_set_name(
		&mut model.ctx,
		&mut model.current_song_name,
		&mut model.current_song,
	);
	model.ctx.sink.play();
}

fn maybe_goto_next_song(model: &mut Model, update_temp: &mut UpdateTemp) {
	let ctx = &mut model.ctx;
	if !ctx.sink.empty() || ctx.playlist.remaining.is_empty() {
		return;
	}

	let first = ctx.playlist.remaining[0].clone();
	try_update_song_duration(ctx, &first);
	ctx.playlist.remaining.remove(0);
	handle_command_return(
		run_macro_or(ctx, "@song_end", &[], ""),
		&mut model.command_output,
		update_temp,
	);

	ctx.playlist.progress = Duration::ZERO;
	if ctx.playlist.remaining.is_empty() {
		remaining_songs_ended(ctx, &mut model.current_song_name);
		handle_command_return(
			run_macro_or(ctx, "@list_end", &[], ""),
			&mut model.command_output,
			update_temp,
		);
	}
	if !ctx.playlist.remaining.is_empty() {
		load_first_song_and_set_name(ctx, &mut model.current_song_name, &mut model.current_song);
		handle_command_return(
			run_macro_or(ctx, "@song_start", &[], ""),
			&mut model.command_output,
			update_temp,
		);
	}
}

fn handle_command_return(
	cmd_return: Result<CommandReturn, impl Error>,
	command_output: &mut String,
	update_temp: &mut UpdateTemp,
) {
	match cmd_return {
		Ok(CommandReturn::Nothing) => (),
		Ok(CommandReturn::Quit) => update_temp.quit = true,
		Ok(CommandReturn::QuitNoSave) => update_temp.quit_no_save = true,
		Ok(CommandReturn::ReloadFirstSong) => update_temp.reload_first_song = true,
		Err(e) => command_output.push_str(&e.to_string()),
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
		let path = if first.is_absolute() || ctx.program_mode == ProgramMode::Temp {
			first.clone()
		} else {
			ctx.files.root.join(&first)
		};
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
