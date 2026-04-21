// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use super::error::Result;
use crate::{Message, serializer};
use ratatui::crossterm::event::KeyCode;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow::Borrowed, collections::HashMap, fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
	/// what to do when songs are playable after not being playable
	pub start_play_state: StartPlayState,
	/// when to save automatically
	pub autosave: AutosavePreference,
	/// the keys you can press in normal mode
	pub keybinds: Vec<(KeyCode, Message)>,
	/// type "m NAME" to run all commands listed under the macro
	#[serde(serialize_with = "serializer::sorted_hashmap")]
	pub macros: HashMap<String, String>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			start_play_state: StartPlayState::Never,
			autosave: AutosavePreference::AfterSeconds(3 * 60),
			macros: HashMap::from(
				[
					("@cmd_empty", ""),
					("@list_end", ""),
					("@song_end", ""),
					("@song_start", ""),
				]
				.map(|(l, r)| (l.to_owned(), r.to_owned())),
			),
			keybinds: vec![
				// general
				(KeyCode::Char(':'), Message::GotoCommandMode),
				(KeyCode::Char(';'), Message::GotoCommandMode),
				(KeyCode::Char('c'), Message::GotoCommandMode),
				(
					KeyCode::Char('?'),
					Message::RunCommand("help first".to_owned()),
				),
				(KeyCode::Char('q'), Message::Quit { save: true }),
				(KeyCode::Char('S'), Message::RunCommand("s".to_owned())),
				// managing this song
				(KeyCode::Char(' '), Message::RunCommand("p".to_owned())),
				(KeyCode::Char('p'), Message::RunCommand("p-".to_owned())),
				(KeyCode::Char('P'), Message::RunCommand("p+".to_owned())),
				(KeyCode::Right, Message::RunCommand("gf 5s".to_owned())),
				(KeyCode::Left, Message::RunCommand("gb 5s".to_owned())),
				(KeyCode::Up, Message::RunCommand("pv+ 5".to_owned())),
				(KeyCode::Down, Message::RunCommand("pv- 5".to_owned())),
				(KeyCode::Char('0'), Message::RunCommand("g".to_owned())),
				// managing this list
				(KeyCode::Char('r'), Message::RunCommand("r".to_owned())),
				(KeyCode::Char('j'), Message::RunCommand("pn".to_owned())),
				(KeyCode::Char('k'), Message::RunCommand("pp".to_owned())),
				// filters and tags
				(KeyCode::Char('f'), Message::StartCommand("fte ".to_owned())),
				(KeyCode::Char('F'), Message::StartCommand("fta ".to_owned())),
				(KeyCode::Char('s'), Message::StartCommand("fs ".to_owned())),
				(KeyCode::Char('t'), Message::StartCommand("tac ".to_owned())),
				(KeyCode::Char('T'), Message::StartCommand("trc ".to_owned())),
				// lists
				(KeyCode::Char('l'), Message::StartCommand("ls ".to_owned())),
				(KeyCode::Char('L'), Message::RunCommand("lg".to_owned())),
				// macros
				(KeyCode::Char('m'), Message::StartCommand("m".to_owned())),
				(KeyCode::Char('M'), Message::RunCommand("ml".to_owned())),
			],
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
	Remember,
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
		pretty_config.indentor = Borrowed("\t");
		pretty_config.new_line = Borrowed("\n");

		let config_string = ron::ser::to_string_pretty(self, pretty_config).map_err(Box::new)?;
		let path = savedata_path.join("config.ron");
		fs::write(path, config_string)?;
		Ok(())
	}
}
