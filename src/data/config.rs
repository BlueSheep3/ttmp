use super::get_savedata_path;
use crate::serializer;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, result};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
	/// the speed of the music
	pub speed: f32,
	/// the volume of the music
	pub volume: f32,
	/// what to do when songs are playable after not being playable
	pub start_play_state: StartPlayState,
	/// whether to always show the current progress,
	/// this does mean it will redraw on every frame
	pub show_song_progress: bool,
	/// the name of the current playlist
	pub current_playlist: String,
	/// type "m NAME" to run all commands listed under the macro
	#[serde(serialize_with = "serializer::sorted_hashmap")]
	pub macros: HashMap<String, String>,
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

impl StartPlayState {
	pub fn should_play(self) -> bool {
		match self {
			Self::Always => true,
			Self::Never => false,
			Self::Remember(play) => play,
		}
	}
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
