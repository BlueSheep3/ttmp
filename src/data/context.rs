// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use super::{
	config::{Config, StartPlayState},
	error::Result,
	files::{FileData, Files},
	playlist::Playlist,
};
use rodio::{OutputStream, OutputStreamBuilder, Sink};
use std::{
	path::{Path, PathBuf},
	time::Duration,
};

pub struct Context {
	pub program_mode: ProgramMode,
	pub cmd_out: String,
	pub config: Config,
	pub files: Files,
	pub playlist: Playlist,
	pub sink: Sink,
	pub savedata_path: PathBuf,

	// these are just here, so the music doesnt stop, due to them being dropped
	_stream_handle: OutputStream,
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
	pub fn new_main(savedata_path: &Path) -> Result<Self> {
		let program_mode = ProgramMode::Main;
		let config = Config::load(savedata_path)?;
		let files = Files::load(savedata_path)?;
		let playlist = Playlist::load(&config.current_playlist, savedata_path)?;
		let mut stream_handle = OutputStreamBuilder::open_default_stream()?;
		stream_handle.log_on_drop(false);
		let sink = Sink::connect_new(stream_handle.mixer());

		let ctx = Self {
			program_mode,
			cmd_out: String::new(),
			config,
			files,
			playlist,
			sink,
			savedata_path: savedata_path.to_owned(),
			_stream_handle: stream_handle,
		};
		ctx.init_sink();
		Ok(ctx)
	}

	pub fn new_temp(file_paths: &[PathBuf], savedata_path: &Path) -> Result<Self> {
		let program_mode = ProgramMode::Temp;
		let mut config = Config::load(savedata_path)?;
		let mut files = Files::load(savedata_path)?;
		let mut playlist = Playlist::default();
		let mut stream_handle = OutputStreamBuilder::open_default_stream()?;
		stream_handle.log_on_drop(false);
		let sink = Sink::connect_new(stream_handle.mixer());

		config.start_play_state = StartPlayState::Always;
		config.current_playlist = "temp".to_owned();
		files.mappings = file_paths
			.iter()
			.map(|f| (f.clone(), FileData::default()))
			.collect();
		playlist.remaining = file_paths.to_vec().into();
		playlist.progress = Duration::ZERO;

		let ctx = Self {
			program_mode,
			cmd_out: String::new(),
			config,
			files,
			playlist,
			sink,
			savedata_path: savedata_path.to_owned(),
			_stream_handle: stream_handle,
		};
		ctx.init_sink();
		Ok(ctx)
	}

	fn init_sink(&self) {
		let should_play = matches!(
			self.config.start_play_state,
			StartPlayState::Always | StartPlayState::Remember(true)
		);
		if should_play && !self.playlist.remaining.is_empty() {
			self.sink.play();
		} else {
			self.sink.pause();
		}
		self.sink.set_speed(self.config.speed);
		self.sink.set_volume(self.config.volume);
	}

	pub fn get_current_duration(&self) -> Option<Duration> {
		let first = self.playlist.remaining.front()?;
		let song = self.files.get(first)?;
		song.duration
	}
}
