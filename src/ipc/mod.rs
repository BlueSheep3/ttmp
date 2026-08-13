// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

mod reader;
mod writer;

pub use reader::FileReader;

use std::{
	error::Error,
	fs::{File, TryLockError},
	io,
	ops::ControlFlow,
	path::PathBuf,
};

#[cfg(target_os = "windows")]
const PIPE_NAME: &str = "//./pipe/ipc_ttmp_xmyuiwqcoecmztrciqenasjkf";
#[cfg(unix)]
const PIPE_NAME: &str = "/tmp/ipc_ttmp_xmyuiwqcoecmztrciqenasjkf";

/// Either sends over the file that you just opened (if you opened any),
/// or starts listening to other processes sending over files.
/// Returns `None` if this process should do no inter process communication.
pub fn send_or_start_listening(
	args_files: &[PathBuf],
	force_ipc: Option<bool>,
) -> Result<ControlFlow<(), Option<FileReader>>, Box<dyn Error>> {
	let enabled = match force_ipc {
		Some(force) => force,
		// if this is not started in the terminal, there will only ever be a single arg.
		// if the file path is relative, this process was most likely
		// manually started in a terminal, in which case we want this to be isolated.
		None => args_files.first().is_some_and(|f| f.is_absolute()),
	};
	if !enabled {
		return Ok(ControlFlow::Continue(None));
	}

	// if another instance is running, send the file and exit
	if !is_only_instance_and_lock()? {
		for file in args_files {
			writer::try_send_to_pipe(PIPE_NAME, file)?;
		}
		return Ok(ControlFlow::Break(()));
	}

	let reader = FileReader::default();
	reader.start_receiving(PIPE_NAME);
	Ok(ControlFlow::Continue(Some(reader)))
}

fn is_only_instance_and_lock() -> Result<bool, io::Error> {
	#[cfg(target_os = "windows")]
	let lock_name = dirs::home_dir()
		.expect("Home Directory not found")
		.join("AppData/Local/Temp/ipc_ttmp_lock_dj72nAk2Xl9cHS11hAXo9Cj455g");
	#[cfg(unix)]
	let lock_name = PathBuf::from("/tmp/ipc_ttmp_lock_dj72nAk2Xl9cHS11hAXo9Cj455g");

	let file = File::create(lock_name)?;

	// if this is not the only instance, another instance will
	// have already aquired this lock, meaning this will fail.
	match file.try_lock() {
		Ok(()) => {
			// do not close the file here,
			// instead the file will automatically be closed when this process dies
			std::mem::forget(file);
			Ok(true)
		}
		Err(TryLockError::WouldBlock) => Ok(false),
		Err(TryLockError::Error(e)) => Err(e),
	}
}
