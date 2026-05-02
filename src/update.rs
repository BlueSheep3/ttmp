// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use crate::{
	Message, Model,
	command::{CommandReturn, match_input, run_macro_or},
	data::{
		config::AutosavePreference,
		context::{Context, ProgramMode},
		files::{FileData, is_mp4_file, make_temp_mp4_copy},
		media,
	},
};
use rodio::{Decoder, Source};
use std::{
	error::Error,
	fs::File,
	io::BufReader,
	path::Path,
	time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn init(model: &mut Model) {
	if model.ctx.playlist.remaining.is_empty() {
		remaining_songs_ended(&mut model.ctx);
	}
	if !model.ctx.playlist.remaining.is_empty() {
		load_first_song(&mut model.ctx);
	}
}

// TEMP temporary hacky solution to handle the CommandReturn values
#[derive(Default)]
struct UpdateTemp {
	quit: bool,
	quit_no_save: bool,
	quit_no_abort: bool,
	reload_first_song: bool,
}

pub fn update(mut model: Box<Model>, message: Message) -> Result<(Box<Model>, Option<Message>)> {
	let mut update_temp = UpdateTemp::default();

	receive_files_over_ipc(&mut model);

	handle_message(&mut model, message, &mut update_temp)?;

	if let Ok(cmd) = model.cmd_receiver.try_recv() {
		run_command(&mut model, &mut update_temp, &cmd)?;
	}

	if !model.ctx.player.is_paused() {
		let elapsed = model.last_update_time.elapsed();
		// if more than 5 seconds passed since the last update,
		// that probably means the computer was in sleep mode,
		// meaning the song wasn't actually playing during that time.
		if elapsed <= Duration::from_secs(5) {
			model.ctx.playlist.progress += elapsed;
		}
	}
	model.last_update_time = Instant::now();

	if let AutosavePreference::AfterSeconds(s) = model.ctx.config.autosave
		&& model.last_autosave_time.elapsed().as_secs() > s as u64
	{
		super::unimportant_maybe_save(&model.ctx);
		model.last_autosave_time = Instant::now();
	}

	let msg = handle_update_temp(&update_temp, &mut model.ctx);

	maybe_goto_next_song(&mut model, &mut update_temp);

	media::common_update();
	if model.last_media_update.elapsed() > Duration::from_secs(5) {
		model.ctx.update_media_progress()?;
		model.last_media_update = Instant::now();
	}

	Ok((model, msg))
}

fn handle_update_temp(update_temp: &UpdateTemp, ctx: &mut Context) -> Option<Message> {
	match () {
		() if update_temp.quit => Some(Message::Quit {
			save: true,
			abort_on_error: true,
		}),
		() if update_temp.quit_no_save => Some(Message::Quit {
			save: false,
			abort_on_error: false,
		}),
		() if update_temp.quit_no_abort => Some(Message::Quit {
			save: true,
			abort_on_error: false,
		}),
		() if update_temp.reload_first_song => {
			load_first_song(ctx);
			Some(Message::DoUpdateAgain)
		}
		() => None,
	}
}

fn handle_message(model: &mut Model, message: Message, update_temp: &mut UpdateTemp) -> Result<()> {
	match message {
		Message::DoUpdateAgain => (),
		Message::GotoNormalMode => {
			model.ctx.cmd_out = String::new();
			model.current_command = None;
		}
		Message::GotoCommandMode => _ = model.current_command.get_or_insert(String::new()),
		Message::ToggleScreenRedraws => model.ctx.state.dont_redraw_screen ^= true,

		Message::Quit { .. } => (), // this gets handled in the main loop
		Message::RunCommand(cmd) => run_command(model, update_temp, &cmd)?,
		Message::StartCommand(cmd) => model.current_command = Some(cmd),

		Message::TypedChar(c) => {
			model.current_command.get_or_insert(String::new()).push(c);
		}
		Message::Backspace => {
			if let Some(cmd) = &mut model.current_command {
				cmd.pop();
			}
		}
		Message::Enter => {
			if let Some(cmd) = &mut model.current_command {
				let cmd_clone = cmd.clone();
				run_command(model, update_temp, &cmd_clone)?;
				model.current_command = None;
			}
		}
	}
	Ok(())
}

fn run_command(model: &mut Model, update_temp: &mut UpdateTemp, cmd: &str) -> Result<()> {
	model.ctx.cmd_out.clear();

	let was_not_empty = !model.ctx.playlist.remaining.is_empty();

	handle_command_return(
		match_input(cmd, &mut model.ctx),
		&mut model.ctx.cmd_out,
		update_temp,
	);

	if model.ctx.playlist.remaining.is_empty() {
		model.ctx.player.pause();
	}

	if was_not_empty && model.ctx.playlist.remaining.is_empty() {
		remaining_songs_ended(&mut model.ctx);
		handle_command_return(
			run_macro_or(&mut model.ctx, "@list_end", &[], ""),
			&mut model.ctx.cmd_out,
			update_temp,
		);
	}
	Ok(())
}

fn receive_files_over_ipc(model: &mut Model) {
	let Some(server) = &model.ipc_server else {
		return;
	};
	let paths = server.drain_file_list();
	if paths.is_empty() {
		return;
	}
	model.ctx.cmd_out.push('\n');
	for path in paths.into_iter().filter(|p| p.is_file()) {
		model.ctx.cmd_out.push_str(&format!(
			"Added Song: {}",
			path.file_name()
				.expect("paths sent over ipc should be valid")
				.to_string_lossy()
		));
		model.ctx.playlist.remaining.push_front(path.clone());
		model.ctx.files.mappings.insert(path, FileData::default());
	}
	model.ctx.playlist.progress = Duration::ZERO;
	load_first_song(&mut model.ctx);
	model.ctx.player.play();
}

fn maybe_goto_next_song(model: &mut Model, update_temp: &mut UpdateTemp) {
	let ctx = &mut model.ctx;
	if !ctx.player.empty() || ctx.playlist.remaining.is_empty() {
		return;
	}

	let first = ctx.playlist.remaining[0].clone();
	try_update_song_duration(ctx, &first);
	ctx.playlist.next_song();
	handle_command_return(
		run_macro_or(ctx, "@song_end", &[], ""),
		&mut ctx.cmd_out,
		update_temp,
	);

	ctx.playlist.progress = Duration::ZERO;
	if ctx.playlist.remaining.is_empty() {
		remaining_songs_ended(ctx);
		handle_command_return(
			run_macro_or(ctx, "@list_end", &[], ""),
			&mut ctx.cmd_out,
			update_temp,
		);
	}

	if matches!(ctx.config.autosave, AutosavePreference::AfterSongFinished) {
		super::unimportant_maybe_save(ctx);
	}

	if !ctx.playlist.remaining.is_empty() {
		load_first_song(ctx);
		handle_command_return(
			run_macro_or(ctx, "@song_start", &[], ""),
			&mut ctx.cmd_out,
			update_temp,
		);
	}
}

fn handle_command_return(
	cmd_return: std::result::Result<CommandReturn, impl Error>,
	command_output: &mut String,
	update_temp: &mut UpdateTemp,
) {
	match cmd_return {
		Ok(CommandReturn::Nothing) => (),
		Ok(CommandReturn::Quit) => update_temp.quit = true,
		Ok(CommandReturn::QuitNoSave) => update_temp.quit_no_save = true,
		Ok(CommandReturn::QuitNoAbort) => update_temp.quit_no_abort = true,
		Ok(CommandReturn::ReloadFirstSong) => update_temp.reload_first_song = true,
		Err(e) => command_output.push_str(&e.to_string()),
	}
}

fn load_first_song(ctx: &mut Context) {
	ctx.player.stop();

	let (file, first) = loop {
		let Some(first) = ctx.playlist.remaining.front().cloned() else {
			// cant call @list_end event here, since we are not in the main loop
			remaining_songs_ended(ctx);
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
			match make_temp_mp4_copy(&path, &ctx.savepaths.data) {
				Ok(p) => path = p,
				Err(_) => {
					ctx.cmd_out += &format!("Failed to convert song to mp3: {}\n", first.display());
					ctx.playlist.next_song();
					continue;
				}
			}
		}
		match File::open(path) {
			Ok(file) => break (file, first),
			Err(_) => {
				ctx.cmd_out += &format!("Failed to load song: {}\n", first.display());
				ctx.playlist.next_song();
				continue;
			}
		};
	};
	let file = BufReader::new(file);

	let mut decoder = Decoder::new(file).expect("unable to convert file to a music file");

	// update the cached duration to be accurate if the decoder type supports it
	if let Some(total) = decoder.total_duration()
		&& let Some(file) = ctx.files.get_mut(&first)
	{
		file.duration = Some(total);
	}

	// `try_seek` is a faster alternative to `skip_duration`,
	// but isn't supported for all file formats
	if decoder.try_seek(ctx.playlist.progress).is_ok() {
		ctx.player.append(decoder);
	} else {
		// logging the error value of `try_seek` would be nice here,
		// but in many cases this would print text over other "ui" elements.
		let source = decoder.skip_duration(ctx.playlist.progress);
		ctx.player.append(source);
	}

	if let Err(e) = ctx
		.update_media_metadata()
		.and_then(|_| ctx.update_media_progress())
	{
		ctx.cmd_out += &format!("failed to update media metadata: {e}\n");
	}
}

fn remaining_songs_ended(ctx: &mut Context) {
	ctx.player.pause();
	ctx.playlist.progress = Duration::ZERO;
}

/// sets the cached duration of `song` to the progress of the current song
fn try_update_song_duration(ctx: &mut Context, song: &Path) {
	if let Some(file) = ctx.files.get_mut(song) {
		file.duration = Some(ctx.playlist.progress);
	}
}
