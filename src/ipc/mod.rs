// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

mod reader;
mod writer;

pub use reader::FileReader;

use std::{env, error::Error, io, ops::ControlFlow, path::PathBuf};

#[cfg(target_os = "windows")]
const PIPE_NAME: &str = "//./pipe/ipc_ttmp_xmyuiwqcoecmztrciqenasjkf";
#[cfg(unix)]
const PIPE_NAME: &str = "/tmp/ipc_ttmp_xmyuiwqcoecmztrciqenasjkf";

// the windows version does not use a lock file
#[cfg(unix)]
const LOCK_NAME: &str = "/tmp/ipc_ttmp_lock_dj72nAk2Xl9cHS11hAXo9Cj455g";

/// Either sends over the file that you just opened (if you opened any),
/// or starts listening to other processes sending over files.
/// Returns `None` if this process should do no inter process communication.
pub fn send_or_start_listening() -> Result<ControlFlow<(), Option<FileReader>>, Box<dyn Error>> {
	// if this is not started in the terminal, there will only ever be a single arg
	let file = env::args_os().nth(1).map(PathBuf::from);

	// if the file path is relative, this process was most likely
	// manually started in a terminal, in which case we want this to be isolated.
	if let Some(file) = file
		&& file.is_absolute()
	{
		// if another instance is running, send the file and exit
		if !is_only_instance()? {
			writer::try_send_to_pipe(PIPE_NAME, file)?;
			return Ok(ControlFlow::Break(()));
		}

		let reader = FileReader::default();
		reader.start_receiving(PIPE_NAME);
		Ok(ControlFlow::Continue(Some(reader)))
	} else {
		Ok(ControlFlow::Continue(None))
	}
}

// has an important side effect on unix, but not on windows
fn is_only_instance() -> Result<bool, io::Error> {
	#[cfg(unix)]
	{
		use std::fs::{File, TryLockError};
		let file = File::create(LOCK_NAME)?;

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

	// on windows, the named pipe will only exist while it's being used,
	// so just checking whether the file exists is enough.
	// we don't need to aquire any lock here.
	#[cfg(target_os = "windows")]
	std::fs::exists(PIPE_NAME)
}
