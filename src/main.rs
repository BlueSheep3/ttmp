#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::infinite_loop)]
#![warn(clippy::use_self)]

mod command;
mod data;
mod duration;
mod handle_event;
mod macros;
mod pause_thread;
mod serializer;
mod shmem_reader;
mod shmem_writer;
mod update;
mod view;

use self::data::context::{Context, ContextError};
use shmem_reader::FileReader;
use std::{
	env,
	error::Error,
	fs,
	ops::ControlFlow,
	path::PathBuf,
	process::ExitCode,
	sync::mpsc::{self, Receiver},
	thread,
	time::{Duration, Instant},
};

fn main() -> ExitCode {
	match fallible_main() {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("{e}");

			// if this is not a terminal, then this window will instantly close after
			// the main function, so we wait for input to allow you to read the error.
			// TODO figure out a way to not read_line when `is_terminal`
			if let Err(e) = std::io::stdin().read_line(&mut String::new()) {
				eprintln!("\nFailed to read line: {e}");
			}

			ExitCode::FAILURE
		}
	}
}

// will always restore the regular screen before returning.
fn fallible_main() -> Result<(), Box<dyn Error>> {
	data::create_default_savedata_if_not_present()?;

	let server = match handle_shared_memory()? {
		ControlFlow::Continue(s) => s,
		ControlFlow::Break(()) => return Ok(()),
	};

	let (sender, receiver) = mpsc::channel();
	let _pause_thread = thread::spawn(move || pause_thread::main(sender));
	let ctx = Context::new_automatic(env::args_os())?;

	let mut terminal = ratatui::try_init()?;
	let mut model = Model::new(ctx, receiver, server);
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
				if save {
					maybe_save(&model.ctx)?;
				}
				return Ok(());
			}

			(model, message) = update::update(model, m)?;
		}
	}
}

struct Model {
	current_command: Option<String>,

	ctx: Context,
	last_update_time: Instant,
	current_song_name: String,
	current_song: PathBuf,

	pause_receiver: Receiver<()>,
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
	fn new(ctx: Context, pause_receiver: Receiver<()>, ipc_server: Option<FileReader>) -> Self {
		Self {
			current_command: None,

			ctx,
			last_update_time: Instant::now(),
			current_song_name: String::new(),
			current_song: PathBuf::new(),

			pause_receiver,
			ipc_server,
		}
	}
}

/// Either sends over the file that you just opened (if you opened any),
/// or starts listening to other processes sending over files.
/// Returns `None` if this process should do no inter process communication.
fn handle_shared_memory() -> Result<ControlFlow<(), Option<FileReader>>, Box<dyn Error>> {
	const PIPE_NAME: &str = "//./pipe/ipc_music_player_xmyuiwqcoecmztrciqenasjkf";

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

fn maybe_save(ctx: &Context) -> Result<(), ContextError> {
	if ctx.program_mode.can_save() {
		ctx.config.save()?;
		ctx.files.save()?;
		ctx.playlist.save(&ctx.config.current_playlist)?;
	}
	Ok(())
}
