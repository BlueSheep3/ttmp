// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use super::{
	config::{Config, StartPlayState},
	error::Result,
	files::{FileData, Files},
	media::{Media, setup_media},
	playlist::Playlist,
	state::State,
};
use crate::cli::SavePaths;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::{path::PathBuf, sync::mpsc::Sender, time::Duration};

pub struct Context {
	pub program_mode: ProgramMode,
	pub cmd_out: String,
	pub config: Config,
	pub state: State,
	pub files: Files,
	pub playlist: Playlist,
	pub player: Player,
	pub savepaths: SavePaths,
	pub media: Option<Media>,

	// this are just here, so the music doesnt stop, due to it being dropped
	_device_sink: MixerDeviceSink,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProgramMode {
	/// The mode used when running the program normally.
	Main,
	/// The mode used when opening individual music files.
	/// In this mode, nothing is saved when quitting normally.
	Temp,
}

impl ProgramMode {
	pub fn can_save(&self) -> bool {
		match self {
			Self::Main => true,
			Self::Temp => false,
		}
	}
}

impl Context {
	pub fn new_main(
		savepaths: SavePaths,
		disable_media: bool,
		cmd_sender: Sender<String>,
	) -> Result<Self> {
		let program_mode = ProgramMode::Main;
		let config = Config::load(&savepaths.config)?;
		let state = State::load(&savepaths.data)?;
		let files = Files::load(&savepaths.data)?;
		let playlist = Playlist::load(&state.current_playlist, &savepaths.data)?;
		let mut device_sink = DeviceSinkBuilder::open_default_sink()?;
		device_sink.log_on_drop(false);
		let player = Player::connect_new(device_sink.mixer());
		let media = match disable_media {
			true => None,
			false => Some(setup_media(cmd_sender)?),
		};

		let mut ctx = Self {
			program_mode,
			cmd_out: String::new(),
			config,
			state,
			files,
			playlist,
			player,
			savepaths,
			media,
			_device_sink: device_sink,
		};
		ctx.init_player();
		ctx.update_media_all()?;
		Ok(ctx)
	}

	pub fn new_temp(
		file_paths: &[PathBuf],
		savepaths: SavePaths,
		disable_media: bool,
		cmd_sender: Sender<String>,
	) -> Result<Self> {
		let program_mode = ProgramMode::Temp;
		let mut config = Config::load(&savepaths.config)?;
		let mut state = State::load(&savepaths.data)?;
		let mut files = Files::load(&savepaths.data)?;
		let mut playlist = Playlist::default();
		let mut device_sink = DeviceSinkBuilder::open_default_sink()?;
		device_sink.log_on_drop(false);
		let player = Player::connect_new(device_sink.mixer());
		let media = match disable_media {
			true => None,
			false => Some(setup_media(cmd_sender)?),
		};

		config.start_play_state = StartPlayState::Always;
		state.current_playlist = "temp".to_owned();
		files.mappings = file_paths
			.iter()
			.map(|f| (f.clone(), FileData::default()))
			.collect();
		playlist.remaining = file_paths.to_vec().into();
		playlist.progress = Duration::ZERO;

		let mut ctx = Self {
			program_mode,
			cmd_out: String::new(),
			config,
			state,
			files,
			playlist,
			player,
			savepaths,
			media,
			_device_sink: device_sink,
		};
		ctx.init_player();
		ctx.update_media_all()?;
		Ok(ctx)
	}

	fn init_player(&self) {
		if self.should_be_playing() && !self.playlist.remaining.is_empty() {
			self.player.play();
		} else {
			self.player.pause();
		}
		self.player.set_speed(self.state.speed);
		self.player.set_volume(self.state.volume);
	}

	pub fn get_current_duration(&self) -> Option<Duration> {
		let first = self.playlist.remaining.front()?;
		let song = self.files.get(first)?;
		song.duration
	}

	pub fn should_be_playing(&self) -> bool {
		match self.config.start_play_state {
			StartPlayState::Always => true,
			StartPlayState::Never => false,
			StartPlayState::Remember => self.state.is_playing,
		}
	}
}
