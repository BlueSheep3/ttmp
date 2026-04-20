// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use super::error::Result;
use crate::serializer;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
	/// the speed of the music
	pub speed: f32,
	/// the volume of the music
	pub volume: f32,
	/// what to do when songs are playable after not being playable
	pub start_play_state: StartPlayState,
	/// when to save automatically
	pub autosave: AutosavePreference,
	/// when this is true, the entire screen is replaced by a simple message,
	/// which removes basically all the logic for redrawing the screen
	pub dont_redraw_screen: bool,
	/// the name of the current playlist
	pub current_playlist: String,
	/// type "m NAME" to run all commands listed under the macro
	#[serde(serialize_with = "serializer::sorted_hashmap")]
	pub macros: HashMap<String, String>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			speed: 1.0,
			volume: 1.0,
			start_play_state: StartPlayState::Never,
			autosave: AutosavePreference::AfterSeconds(3 * 60),
			dont_redraw_screen: false,
			current_playlist: "main".to_owned(),
			macros: HashMap::from(
				[
					("@cmd_empty", ""),
					("@list_end", ""),
					("@song_end", ""),
					("@song_start", ""),
				]
				.map(|(l, r)| (l.to_owned(), r.to_owned())),
			),
		}
	}
}

/// Represents what should happen when the program starts being able to play songs again.
/// This can happen for example when the program is opened, or when the playlist
/// is filled with songs after being empty.
#[derive(Serialize, Deserialize, Debug)]
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
	pub fn should_play(&self) -> bool {
		match self {
			Self::Always => true,
			Self::Never => false,
			Self::Remember(play) => *play,
		}
	}
}

/// Represents when you want to save automatically.
#[derive(Serialize, Deserialize, Debug)]
pub enum AutosavePreference {
	/// never autosave
	Never,
	/// autosave every `N` seconds
	AfterSeconds(u32),
	/// autosave whenever you finish a song
	AfterSongFinished,
}

impl Config {
	pub fn load(savedata_path: &Path) -> Result<Self> {
		let path = savedata_path.join("config.ron");
		let config_string = fs::read_to_string(path)?;
		let config = ron::from_str(&config_string).map_err(Box::new)?;
		Ok(config)
	}

	pub fn save(&self, savedata_path: &Path) -> Result<()> {
		let mut pretty_config = PrettyConfig::new();
		pretty_config.indentor = Cow::Borrowed("\t");
		pretty_config.new_line = Cow::Borrowed("\n");

		let config_string = ron::ser::to_string_pretty(self, pretty_config).map_err(Box::new)?;
		let path = savedata_path.join("config.ron");
		fs::write(path, config_string)?;
		Ok(())
	}
}
