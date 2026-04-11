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
mod macros;
mod serializer;
mod shmem_reader;
mod shmem_writer;
mod update;
mod view;

use self::data::{context::Context, error::DataError};
use shmem_reader::FileReader;
use std::{
	env,
	error::Error,
	fs,
	ops::ControlFlow,
	path::PathBuf,
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

	data::create_default_savedata_if_not_present(&cli_args.savedata_path)?;

	let server = if cli_args.disable_ipc {
		None
	} else {
		match handle_shared_memory()? {
			ControlFlow::Continue(s) => s,
			ControlFlow::Break(()) => return Ok(()),
		}
	};

	let (cmd_sender, cmd_receiver) = mpsc::channel();
	let ctx = match cli_args.program_mode {
		data::context::ProgramMode::Main => {
			Context::new_main(&cli_args.savedata_path, cli_args.disable_media, cmd_sender)?
		}
		data::context::ProgramMode::Temp => Context::new_temp(
			&cli_args.files,
			&cli_args.savedata_path,
			cli_args.disable_media,
			cmd_sender,
		)?,
	};

	let mut terminal = ratatui::try_init()?;
	let mut model = Model::new(ctx, cmd_receiver, server);
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
			if let Message::Quit { save } = m {
				return cleanup(model, save);
			}
			(model, message) = update::update(model, m)?;
		}
	}
}

fn cleanup(model: Model, save: bool) -> Result<(), Box<dyn Error>> {
	if save {
		maybe_save(&model.ctx)?;
	}

	// For some reason, detaching the media_controls can take a long time.
	// This detach will automatically happen if media_controls gets dropped.
	// To get around these we detach them on a different thread.
	// This probably causes them to not properly get detached, because the program
	// exits immediatly after this, but i haven't noticed any problems so far.
	if let Some(mut media) = model.ctx.media {
		std::thread::spawn(move || media.controls.detach().ok()); // ignores errors
	}

	Ok(())
}

struct Model {
	current_command: Option<String>,

	ctx: Context,
	last_update_time: Instant,

	cmd_receiver: Receiver<String>,
	last_media_update: Instant,
	ipc_server: Option<FileReader>,
}

#[derive(Debug)]
enum Message {
	DoUpdateAgain,
	GotoNormalMode,
	GotoCommandMode,
	ToggleScreenRedraws,

	Quit { save: bool },
	RunCommand(&'static str),
	StartCommand(&'static str),

	TypedChar(char),
	Backspace,
	Enter,
}

impl Model {
	fn new(ctx: Context, cmd_receiver: Receiver<String>, ipc_server: Option<FileReader>) -> Self {
		Self {
			current_command: None,

			ctx,
			last_update_time: Instant::now(),

			cmd_receiver,
			last_media_update: Instant::now(),
			ipc_server,
		}
	}
}

/// Either sends over the file that you just opened (if you opened any),
/// or starts listening to other processes sending over files.
/// Returns `None` if this process should do no inter process communication.
fn handle_shared_memory() -> Result<ControlFlow<(), Option<FileReader>>, Box<dyn Error>> {
	const PIPE_NAME: &str = "//./pipe/ipc_ttmp_xmyuiwqcoecmztrciqenasjkf";

	// if this is not started in the terminal, there will only ever be a single arg
	let file = env::args_os().nth(1).map(PathBuf::from);

	// if the file path is relative, this process was most likely
	// manually started in a terminal, in which case we want this to be isolated.
	if let Some(file) = file
		&& file.is_absolute()
	{
		let file = file.canonicalize()?;

		// if another instance is running, send the file and exit
		if fs::exists(PIPE_NAME)? {
			shmem_writer::try_send_to_pipe(PIPE_NAME, file)?;
			return Ok(ControlFlow::Break(()));
		}

		let reader = FileReader::default();
		reader.start_receiving(PIPE_NAME);
		Ok(ControlFlow::Continue(Some(reader)))
	} else {
		Ok(ControlFlow::Continue(None))
	}
}

fn maybe_save(ctx: &Context) -> Result<(), DataError> {
	if ctx.program_mode.can_save() {
		ctx.config.save(&ctx.savedata_path)?;
		ctx.files.save(&ctx.savedata_path)?;
		ctx.playlist
			.save(&ctx.config.current_playlist, &ctx.savedata_path)?;
	}
	Ok(())
}
