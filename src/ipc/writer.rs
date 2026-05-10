// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use std::{error::Error, path::Path};

pub fn try_send_to_pipe(pipe_name: &str, file_path: &Path) -> Result<(), Box<dyn Error>> {
	#[cfg(target_os = "windows")]
	{
		use std::{
			fs::OpenOptions,
			io::{BufWriter, Write as _},
		};

		let file_path = file_path.as_os_str().as_encoded_bytes();
		let file = OpenOptions::new().write(true).open(pipe_name)?;

		eprintln!("sending over: {:?}", String::from_utf8_lossy(file_path));

		let mut writer = BufWriter::new(file);
		writer.write_all(file_path)?;
		writer.flush()?;
	}

	#[cfg(unix)]
	{
		use std::{io::Write as _, os::unix::net::UnixStream};
		let mut stream = UnixStream::connect(pipe_name)?;
		stream.write_all(file_path.as_os_str().as_encoded_bytes())?;
	}

	Ok(())
}
