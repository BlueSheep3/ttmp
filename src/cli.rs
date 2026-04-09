use self::error::CliError;
use crate::data::context;
use std::{env, path::PathBuf, process};

pub struct ParsedCommandLineArgs {
	pub program_mode: context::ProgramMode,
	pub files: Vec<PathBuf>,
	pub disable_ipc: bool,
	pub savedata_path: PathBuf,
}

pub fn parse_command_line_args() -> Result<ParsedCommandLineArgs, CliError> {
	let mut program_mode = None;
	let mut files = Vec::new();
	let mut disable_ipc = false;
	let mut savedata_path = None;

	let mut args = env::args_os();
	// the first argument is always the program itself
	args.next();

	while let Some(arg) = args.next() {
		let bytes = arg.as_encoded_bytes();
		if bytes.starts_with(b"-") {
			match bytes {
				b"--help" | b"-h" | b"?" => print_help_and_exit(),
				b"--version" | b"-v" => print_version_and_exit(),
				b"--no-ipc" => disable_ipc = true,
				b"--mode" | b"-m" => {
					let mode = args.next().ok_or(CliError::NoModeSpecifier)?;
					match mode.as_encoded_bytes() {
						b"main" | b"m" => program_mode = Some(context::ProgramMode::Main),
						b"temp" | b"t" => program_mode = Some(context::ProgramMode::Temp),
						_ => return Err(CliError::UnkownMode(mode)),
					}
				}
				b"--path" | b"-p" => {
					let path = args.next().ok_or(CliError::NoSavedataPathSpecifier)?;
					savedata_path = Some(PathBuf::from(path));
				}
				b"--" => {
					files.extend(args.map(PathBuf::from));
					break;
				}
				_ => return Err(CliError::UnknownArg(arg)),
			}
		} else {
			files.push(PathBuf::from(arg));
		}
	}

	let program_mode = program_mode.unwrap_or({
		if files.is_empty() {
			context::ProgramMode::Main
		} else {
			context::ProgramMode::Temp
		}
	});
	let savedata_path = match savedata_path {
		Some(p) => p,
		// this function could produce an error, so only call it when
		// the savedata path hasn't already been specified explicitly
		None => get_savedata_path()?,
	};
	let parsed_args = ParsedCommandLineArgs {
		program_mode,
		files,
		disable_ipc,
		savedata_path,
	};
	Ok(parsed_args)
}

fn print_help_and_exit() -> ! {
	println!(
		"\
A minimal TUI based music player.

All arguments that do not start with a '-' will be interpreted as file paths.
If you specify at least 1 file, you will (unless specified otherwise) start in 'temp' mode
with all of those songs in the current playlist.
Otherwise, you will start in 'main' mode, restoring your previous main mode playlist.

Arguments:
--help, -h         - print help and exit
--version, -v      - print version info and exit
--no-ipc           - disable all interprocess communication
--mode, -m  MODE   - specify what program mode to start in (either 'main' or 'temp')
--path, -p  PATH   - specify the savedata path (this should be a directory)
                     note that default savedata will only be created if PATH doesn't exist
--                 - force all arguments after this one to be interpreted as file paths
"
	);
	process::exit(0);
}

fn print_version_and_exit() -> ! {
	const NAME: &str = env!("CARGO_PKG_NAME");
	const VERSION: &str = env!("CARGO_PKG_VERSION");
	println!("{NAME} {VERSION}");
	std::process::exit(0);
}

fn get_savedata_path() -> Result<PathBuf, CliError> {
	let config = dirs::data_dir().ok_or(CliError::CantFindSavedataPath)?;
	let path = config.join("musicplayer");
	Ok(path)
}

pub mod error {
	use std::ffi::OsString;
	use thiserror::Error;

	#[derive(Error, Debug)]
	pub enum CliError {
		#[error("unknown argument: {0:?}")]
		UnknownArg(OsString),
		#[error("unkown mode: {0:?}\nmust be either 'main' or 'temp'")]
		UnkownMode(OsString),
		#[error("no mode was specified. --mode requires 1 argument")]
		NoModeSpecifier,
		#[error("no savedata path was specified. --path requires 1 argument")]
		NoSavedataPathSpecifier,
		#[error("can't find savedata path")]
		CantFindSavedataPath,
	}
}
