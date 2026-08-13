// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::infinite_loop)]
#![warn(clippy::use_self)]

mod cli;
mod command;
mod data;
mod duration;
mod handle_event;
mod ipc;
mod macros;
mod serializer;
mod update;
mod view;

use self::data::{context::Context, error::DataError};
use serde::{Deserialize, Serialize};
use std::{
	error::Error,
	ops::ControlFlow,
	process::ExitCode,
	sync::mpsc::{self, Receiver},
	time::{Duration, Instant},
};

fn main() -> ExitCode {
	match fallible_main() {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("{e}");
			ExitCode::FAILURE
		}
	}
}

// will always restore the regular screen before returning.
fn fallible_main() -> Result<(), Box<dyn Error>> {
	let cli_args = cli::parse_command_line_args()?;

	data::create_default_savedata_if_not_present(&cli_args.paths)?;

	let server = match ipc::send_or_start_listening(&cli_args.files, cli_args.force_ipc)? {
		ControlFlow::Continue(s) => s,
		ControlFlow::Break(()) => return Ok(()),
	};

	let (cmd_sender, cmd_receiver) = mpsc::channel();
	let ctx = match cli_args.program_mode {
		data::context::ProgramMode::Main => {
			Context::new_main(cli_args.paths, cli_args.disable_media, cmd_sender)?
		}
		data::context::ProgramMode::Temp => Context::new_temp(
			&cli_args.files,
			cli_args.paths,
			cli_args.disable_media,
			cmd_sender,
		)?,
	};

	let mut terminal = ratatui::try_init()?;
	let mut model = Box::new(Model::new(ctx, cmd_receiver, server));
	defer! { ratatui::restore(); }

	update::init(&mut model);

	loop {
		terminal.draw(|f| view::view(&model, f))?;

		let event = ratatui::crossterm::event::poll(Duration::from_millis(300))?;
		let mut message = if event {
			let event = ratatui::crossterm::event::read()?;
			handle_event::handle_event(&model, event)
		} else {
			Some(Message::DoUpdateAgain)
		};

		while let Some(m) = message {
			if let Message::Quit {
				save,
				abort_on_error,
			} = m
			{
				if save
					&& model.ctx.program_mode.can_save()
					&& let Err(e) = important_force_save(&model.ctx)
					&& abort_on_error
				{
					model.ctx.cmd_out += &format!("error while saving on quit:\n{e}");
					// saving failed while trying to quit, so abort the quit,
					// giving you another chance at saving
					break;
				}
				cleanup(model);
				return Ok(());
			}
			(model, message) = update::update(model, m)?;
		}
	}
}

fn cleanup(model: Box<Model>) {
	// For some reason, detaching the media_controls can take a long time.
	// This detach will automatically happen if media_controls gets dropped.
	// To get around these we detach them on a different thread.
	// This probably causes them to not properly get detached, because the program
	// exits immediatly after this, but i haven't noticed any problems so far.
	if let Some(mut media) = model.ctx.media {
		std::thread::spawn(move || media.controls.detach().ok()); // ignores errors
	}
}

struct Model {
	current_command: Option<String>,

	ctx: Context,
	last_update_time: Instant,
	last_autosave_time: Instant,

	cmd_receiver: Receiver<String>,
	last_media_update: Instant,
	ipc_server: Option<ipc::FileReader>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Message {
	DoUpdateAgain,
	GotoNormalMode,
	GotoCommandMode,
	ToggleScreenRedraws,

	Quit { save: bool, abort_on_error: bool },
	RunCommand(String),
	StartCommand(String),

	TypedChar(char),
	Backspace,
	Enter,
}

impl Model {
	fn new(
		ctx: Context,
		cmd_receiver: Receiver<String>,
		ipc_server: Option<ipc::FileReader>,
	) -> Self {
		Self {
			current_command: None,

			ctx,
			last_update_time: Instant::now(),
			last_autosave_time: Instant::now(),

			cmd_receiver,
			last_media_update: Instant::now(),
			ipc_server,
		}
	}
}

/// Will only save in a program mode that can save.
/// If any file fails to be saved, the error is ignored and the other files are still attempted.
fn unimportant_maybe_save(ctx: &Context) {
	if ctx.program_mode.can_save() {
		_ = ctx.config.save(&ctx.savepaths.config);
		_ = ctx.state.save(&ctx.savepaths.data);
		_ = ctx.files.save(&ctx.savepaths.data);
		_ = ctx
			.playlist
			.save(&ctx.state.current_playlist, &ctx.savepaths.data);
	}
}

/// Will always save, no matter what program mode you are in.
/// If any file fails to be saved, the other files will still try to save,
/// and then all errors are collected.
fn important_force_save(ctx: &Context) -> Result<(), DataError> {
	let results = [
		ctx.config.save(&ctx.savepaths.config),
		ctx.state.save(&ctx.savepaths.data),
		ctx.files.save(&ctx.savepaths.data),
		ctx.playlist
			.save(&ctx.state.current_playlist, &ctx.savepaths.data),
	];
	let errors = results
		.into_iter()
		.filter_map(|r| r.err())
		.collect::<Vec<_>>();
	if errors.is_empty() {
		Ok(())
	} else {
		Err(DataError::MultiError(errors))
	}
}
