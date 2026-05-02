// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

pub mod config;
pub mod context;
pub mod files;
pub mod media;
pub mod playlist;
pub mod state;

use self::{error::Result, state::State};
use crate::{
	cli::SavePaths,
	data::{config::Config, files::Files, playlist::Playlist},
};
use std::{fs, path::PathBuf};

pub fn create_default_savedata_if_not_present(paths: &SavePaths) -> Result<()> {
	if !fs::exists(paths.config.join("config.ron"))? {
		println!("No config found, creating new default config...");
		fs::create_dir_all(&paths.config)?;
		Config::default().save(&paths.config)?;
	}

	// i need an instance of state, since the current playlist must exist
	let state;
	if !fs::exists(paths.data.join("state.ron"))? {
		println!("No state found, creating new default state...");
		fs::create_dir_all(&paths.data)?;
		state = State::default();
		state.save(&paths.data)?;
	} else {
		state = State::load(&paths.data)?;
	}

	if !fs::exists(paths.data.join("files.ron"))? {
		println!("No files found, creating new default files...");
		fs::create_dir_all(&paths.data)?;
		let music = match dirs::audio_dir() {
			Some(m) => m,
			None => {
				println!(
					"\
Can't find a default Music folder.
Give the path of the folder that contains all your Music.\
"
				);
				loop {
					print!("Music Path: ");
					let music_path = readln();
					let music_path = PathBuf::from(music_path);
					if music_path.is_dir() {
						break music_path;
					}
					println!("The path you provided is not a directory that exists.");
				}
			}
		};
		Files::empty_with_root(music).save(&paths.data)?;
	}

	let current_list_path = paths
		.data
		.join("list")
		.join(format!("{}.ron", state.current_playlist));
	if !fs::exists(current_list_path)? {
		println!("No playlist found, creating new default playlist...");
		fs::create_dir_all(paths.data.join("list"))?;
		Playlist::default().save(&state.current_playlist, &paths.data)?;
	}

	Ok(())
}

fn readln() -> String {
	let mut input = String::new();
	std::io::stdin()
		.read_line(&mut input)
		.expect("reading from stdin failed");

	if input.ends_with('\n') {
		input.pop();
	}
	if input.ends_with('\r') {
		input.pop();
	}

	input
}

pub mod error {
	use rodio::DeviceSinkError;
	use std::{io, result};
	use thiserror::Error;

	pub type Result<T> = result::Result<T, DataError>;

	#[derive(Error, Debug)]
	pub enum DataError {
		#[error("io error: {0}")]
		Io(#[from] io::Error),
		#[error("device sink error: {0}")]
		DeviceSink(#[from] DeviceSinkError),
		#[error("media controls error: {0}")]
		Souvlaki(#[from] souvlaki::Error),

		// these are wrapped in Box, because SpannedError is 88 bytes and Error is 72 bytes
		#[error("ron spanned error: {0}")]
		RonSpanned(#[from] Box<ron::error::SpannedError>),
		#[error("ron error: {0}")]
		Ron(#[from] Box<ron::Error>),

		#[error("the file name {0:?} is not valid utf8")]
		FileNotUtf8Name(std::ffi::OsString),

		#[error("{}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"))]
		MultiError(Vec<Self>),
	}
}
