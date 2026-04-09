// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use std::{
	error::Error,
	fs::OpenOptions,
	io::{BufWriter, Write},
	path::PathBuf,
};

pub fn try_send_to_pipe(pipe_name: &str, file_path: PathBuf) -> Result<(), Box<dyn Error>> {
	let file_path = file_path.to_str().ok_or("invalid utf8")?;
	let file = OpenOptions::new().write(true).open(pipe_name)?;

	let mut writer = BufWriter::new(file);
	writer.write_all(file_path.as_bytes())?;
	writer.flush()?;

	Ok(())
}
