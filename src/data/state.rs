// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use super::error::Result;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow::Borrowed, fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct State {
	/// the speed of the music
	pub speed: f32,
	/// the volume of the music
	pub volume: f32,
	/// was the song playing while you saved? (as opposed to being paused)
	pub is_playing: bool,
	/// when this is true, the entire screen is replaced by a simple message,
	/// which removes basically all the logic for redrawing the screen
	pub dont_redraw_screen: bool,
	/// the name of the current playlist
	pub current_playlist: String,
}

impl Default for State {
	fn default() -> Self {
		Self {
			speed: 1.0,
			volume: 1.0,
			is_playing: false,
			dont_redraw_screen: false,
			current_playlist: "main".to_owned(),
		}
	}
}

impl State {
	pub fn load(savedata_path: &Path) -> Result<Self> {
		let path = savedata_path.join("state.ron");
		let state_string = fs::read_to_string(path)?;
		let state = ron::from_str(&state_string).map_err(Box::new)?;
		Ok(state)
	}

	pub fn save(&self, savedata_path: &Path) -> Result<()> {
		let mut pretty_config = PrettyConfig::new();
		pretty_config.indentor = Borrowed("\t");
		pretty_config.new_line = Borrowed("\n");

		let state_string = ron::ser::to_string_pretty(self, pretty_config).map_err(Box::new)?;
		let path = savedata_path.join("state.ron");
		fs::write(path, state_string)?;
		Ok(())
	}
}
