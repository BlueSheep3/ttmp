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
	/// the folder that all the files are in
	pub parent_path: PathBuf,
	/// how far you are into the current song
	pub current_progress: Duration,
	/// the speed of the music
	pub speed: f32,
	/// the volume of the music
	pub volume: f32,
	/// whether the music should start playing as soon as the program starts
	pub start_playing_immediately: bool,
	/// type "m NAME" to run all commands listed under the macro
	#[serde(serialize_with = "serializer::sorted_hashmap")]
	pub macros: HashMap<String, String>,
	/// the remaining songs in order (current song included)
	pub remaining: Vec<PathBuf>,
	/// the songs to loop when `remaining` ends
	pub looping_songs: Vec<PathBuf>,
	/// all music files, paths should be relative to the parent folder
	#[serde(serialize_with = "serializer::sorted_hashmap")]
	pub files: HashMap<PathBuf, FileData>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FileData {
	#[serde(serialize_with = "serializer::sorted_hashset")]
	pub tags: HashSet<String>,
}

pub fn load() -> Result<Config> {
	let config_string = fs::read_to_string(get_config_path())?;
	let config = ron::from_str(&config_string)?;
	Ok(config)
}

impl Config {
	pub fn save(&self) -> Result<()> {
		let mut pretty_config = PrettyConfig::new();
		pretty_config.indentor = "\t".to_owned();
		pretty_config.new_line = "\n".to_owned();

		let config_string = ron::ser::to_string_pretty(self, pretty_config)?;
		fs::write(get_config_path(), config_string)?;
		Ok(())
	}

	/// add all files that are in the system, but not in the config,
	/// and remove all files that are in the config but not in the system.
	pub fn reload_files(&mut self) -> Result<()> {
		let system_files = get_all_files_in(&self.parent_path)?;

		for full_path in &system_files {
			let rel_path = full_path
				.strip_prefix(&self.parent_path)
				.unwrap_or_else(|_| unreachable!())
				.to_path_buf();
			self.files.entry(rel_path).or_default();
		}

		let mut files_to_remove = Vec::new();

		for rel_path in self.files.keys() {
			let full_path = self.parent_path.join(rel_path);
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

fn get_config_path() -> PathBuf {
	#[cfg(not(target_os = "windows"))]
	compile_error!("get_config_path() only works on windows");

	let path = std::env::var("APPDATA").expect("APPDATA not found");
	let path = PathBuf::from(path);
	let path = path.parent().expect("appdata doesn't have a parent");
	path.join("LocalLow/BlueSheep3/Music Player/config.ron")
}

type Result<T> = result::Result<T, ConfigError>;

#[derive(Error, Debug)]
pub enum ConfigError {
	#[error("io error: {0}")]
	Io(#[from] io::Error),
	#[error("ron spanned error: {0}")]
	RonSpanned(#[from] ron::error::SpannedError),
	#[error("ron error: {0}")]
	Ron(#[from] ron::Error),
}
