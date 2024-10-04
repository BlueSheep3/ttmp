use super::get_savedata_path;
use crate::serializer;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet},
	fs, io,
	path::{Path, PathBuf},
	result,
	time::Duration,
};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
	/// the folder that all the files are in,
	/// or the single file that is currently playing
	pub path: PathBuf,
	/// the speed of the music
	pub speed: f32,
	/// the volume of the music
	pub volume: f32,
	/// what to do when songs are playable after not being playable
	pub start_play_state: StartPlayState,
	/// whether to always show the current progress,
	/// this does mean it will redraw on every frame
	pub show_song_progress: bool,
	/// the current playlist used in the [Main MetaMode](super::context::MetaMode::Main)
	pub main_playlist: String,
	/// type "m NAME" to run all commands listed under the macro
	#[serde(serialize_with = "serializer::sorted_hashmap")]
	pub macros: HashMap<String, String>,
	/// all music files, paths should be relative to the parent folder
	#[serde(serialize_with = "serializer::sorted_hashmap")]
	pub files: HashMap<PathBuf, FileData>,
}

/// Represents what should happen when the program starts being able to play songs again.
/// This can happen for example when the program is opened, or when the playlist
/// is filled with songs after being empty.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartPlayState {
	/// always start playing when opening the program
	Always,
	/// never start playing when opening the program,
	/// meaning you have to start it manually
	Never,
	/// remember whether the song was playing when you closed the program
	Remember(bool),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FileData {
	#[serde(serialize_with = "serializer::sorted_hashset")]
	pub tags: HashSet<String>,
	/// a cache of how long the song is.
	/// this gets updated when a song finishes playing.
	pub duration: Option<Duration>,
}

impl Config {
	pub fn load() -> Result<Self> {
		let path = get_savedata_path().join("config.ron");
		let config_string = fs::read_to_string(path)?;
		let config = ron::from_str(&config_string).map_err(Box::new)?;
		Ok(config)
	}

	pub fn save(&self) -> Result<()> {
		let mut pretty_config = PrettyConfig::new();
		pretty_config.indentor = "\t".to_owned();
		pretty_config.new_line = "\n".to_owned();

		let config_string = ron::ser::to_string_pretty(self, pretty_config).map_err(Box::new)?;
		let path = get_savedata_path().join("config.ron");
		fs::write(path, config_string)?;
		Ok(())
	}

	/// add all files that are in the system, but not in the config,
	/// and remove all files that are in the config but not in the system.
	pub fn reload_files(&mut self) -> Result<()> {
		let system_files = get_all_files_in(&self.path)?;

		for full_path in &system_files {
			let rel_path = full_path
				.strip_prefix(&self.path)
				.unwrap_or_else(|_| unreachable!())
				.to_path_buf();
			self.files.entry(rel_path).or_default();
		}

		let mut files_to_remove = Vec::new();

		for rel_path in self.files.keys() {
			let full_path = self.path.join(rel_path);
			if !system_files.contains(&full_path) {
				files_to_remove.push(rel_path.clone());
			}
		}
		for rel_path in files_to_remove {
			self.files.remove(&rel_path);
		}

		Ok(())
	}
}

/// gets all files in a folder, including subfolders
fn get_all_files_in(path: &Path) -> result::Result<Vec<PathBuf>, io::Error> {
	let mut files = vec![];

	for entry in fs::read_dir(path)? {
		let entry = entry?;
		if entry.file_type()?.is_dir() {
			files.extend(get_all_files_in(&entry.path())?);
		} else {
			let name = entry.file_name();
			let is_music = name.to_str().map(is_music_file).unwrap_or(false);
			if is_music {
				files.push(entry.path());
			}
		}
	}
	Ok(files)
}

fn is_music_file(file_name: &str) -> bool {
	[".mp3", ".wav", ".ogg" /*, ".mp4"*/]
		.into_iter()
		.any(|end| file_name.ends_with(end))
}

type Result<T> = result::Result<T, ConfigError>;

#[derive(Error, Debug)]
pub enum ConfigError {
	#[error("io error: {0}")]
	Io(#[from] io::Error),

	// these are wrapped in Box, because SpannedError is 88 bytes and Error is 72 bytes
	#[error("ron spanned error: {0}")]
	RonSpanned(#[from] Box<ron::error::SpannedError>),
	#[error("ron error: {0}")]
	Ron(#[from] Box<ron::Error>),
}
