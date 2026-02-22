pub mod config;
pub mod context;
pub mod files;
pub mod playlist;

use self::error::{DataError, Result};
use crate::data::{config::Config, files::Files, playlist::Playlist};
use std::{fs, path::PathBuf};

pub fn create_default_savedata_if_not_present() -> Result<()> {
	let path = get_savedata_path()?;

	if fs::exists(&path)? {
		// already has the savedata, so we do nothing
		return Ok(());
	}

	fs::create_dir_all(path.join("list"))?;

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

	Config::default().save()?;
	Files::empty_with_root(music).save()?;
	Playlist::default().save("main")?;
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

fn get_savedata_path() -> Result<PathBuf> {
	let config = dirs::config_dir().ok_or(DataError::CantFindSavedataPath)?;
	let path = config.join("musicplayer");
	Ok(path)
}

pub mod error {
	use rodio::StreamError;
	use std::{io, result};
	use thiserror::Error;

	pub type Result<T> = result::Result<T, DataError>;

	#[derive(Error, Debug)]
	pub enum DataError {
		#[error("io error: {0}")]
		Io(#[from] io::Error),
		#[error("{0}")]
		Stream(#[from] StreamError),

		// these are wrapped in Box, because SpannedError is 88 bytes and Error is 72 bytes
		#[error("ron spanned error: {0}")]
		RonSpanned(#[from] Box<ron::error::SpannedError>),
		#[error("ron error: {0}")]
		Ron(#[from] Box<ron::Error>),

		#[error("the file name {0:?} is not valid utf8")]
		FileNotUtf8Name(std::ffi::OsString),
		#[error("can't find savedata path")]
		CantFindSavedataPath,
	}
}
