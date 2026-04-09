// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

pub mod config;
pub mod context;
pub mod files;
pub mod playlist;

use self::error::Result;
use crate::data::{config::Config, files::Files, playlist::Playlist};
use std::{
	fs,
	path::{Path, PathBuf},
};

pub fn create_default_savedata_if_not_present(savedata_path: &Path) -> Result<()> {
	if fs::exists(savedata_path)? {
		// already has the savedata, so we do nothing
		return Ok(());
	}
	println!("No savedata found, creating new default savedata...");

	fs::create_dir_all(savedata_path.join("list"))?;

	let music = match dirs::audio_dir() {
		Some(m) => m,
		None => {
			println!(
				"\
Cant find a default Music folder.
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

	Config::default().save(savedata_path)?;
	Files::empty_with_root(music).save(savedata_path)?;
	Playlist::default().save("main", savedata_path)?;
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

		// these are wrapped in Box, because SpannedError is 88 bytes and Error is 72 bytes
		#[error("ron spanned error: {0}")]
		RonSpanned(#[from] Box<ron::error::SpannedError>),
		#[error("ron error: {0}")]
		Ron(#[from] Box<ron::Error>),

		#[error("the file name {0:?} is not valid utf8")]
		FileNotUtf8Name(std::ffi::OsString),
	}
}
