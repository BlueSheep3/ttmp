// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use super::{
	config::{Config, StartPlayState},
	error::Result,
	files::{FileData, Files},
	playlist::{self, Playlist},
};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use souvlaki::{
	MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
	SeekDirection,
};
use std::{
	borrow::Cow::{Borrowed, Owned},
	path::{Path, PathBuf},
	sync::mpsc::Sender,
	time::Duration,
};

pub struct Context {
	pub program_mode: ProgramMode,
	pub cmd_out: String,
	pub config: Config,
	pub files: Files,
	pub playlist: Playlist,
	pub player: Player,
	pub savedata_path: PathBuf,
	pub media_controls: MediaControls,

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
	pub fn new_main(savedata_path: &Path, cmd_sender: Sender<String>) -> Result<Self> {
		let program_mode = ProgramMode::Main;
		let config = Config::load(savedata_path)?;
		let files = Files::load(savedata_path)?;
		let playlist = Playlist::load(&config.current_playlist, savedata_path)?;
		let mut device_sink = DeviceSinkBuilder::open_default_sink()?;
		device_sink.log_on_drop(false);
		let player = Player::connect_new(device_sink.mixer());
		let media_controls = setup_media_controls(cmd_sender)?;

		let mut ctx = Self {
			program_mode,
			cmd_out: String::new(),
			config,
			files,
			playlist,
			player,
			savedata_path: savedata_path.to_owned(),
			media_controls,
			_device_sink: device_sink,
		};
		ctx.init_player();
		ctx.update_media_all()?;
		Ok(ctx)
	}

	pub fn new_temp(
		file_paths: &[PathBuf],
		savedata_path: &Path,
		cmd_sender: Sender<String>,
	) -> Result<Self> {
		let program_mode = ProgramMode::Temp;
		let mut config = Config::load(savedata_path)?;
		let mut files = Files::load(savedata_path)?;
		let mut playlist = Playlist::default();
		let mut device_sink = DeviceSinkBuilder::open_default_sink()?;
		device_sink.log_on_drop(false);
		let player = Player::connect_new(device_sink.mixer());
		let media_controls = setup_media_controls(cmd_sender)?;

		config.start_play_state = StartPlayState::Always;
		config.current_playlist = "temp".to_owned();
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
			files,
			playlist,
			player,
			savedata_path: savedata_path.to_owned(),
			media_controls,
			_device_sink: device_sink,
		};
		ctx.init_player();
		ctx.update_media_all()?;
		Ok(ctx)
	}

	fn init_player(&self) {
		let should_play = matches!(
			self.config.start_play_state,
			StartPlayState::Always | StartPlayState::Remember(true)
		);
		if should_play && !self.playlist.remaining.is_empty() {
			self.player.play();
		} else {
			self.player.pause();
		}
		self.player.set_speed(self.config.speed);
		self.player.set_volume(self.config.volume);
	}

	// this REALLY doesn't like it if you send too many metadata updates too quickly, so use
	// the smaller functions when possible and space out any calls you need to do repeatedly.
	pub fn update_media_all(&mut self) -> Result<()> {
		self.update_media_metadata()?;
		self.update_media_volume()?;
		self.update_media_progress()?;
		Ok(())
	}

	pub fn update_media_metadata(&mut self) -> Result<()> {
		let song_name = self
			.playlist
			.remaining
			.front()
			.map(|f| playlist::get_song_name(f));
		let duration = self.get_current_duration();

		self.media_controls.set_metadata(MediaMetadata {
			title: song_name.as_deref(),
			duration,
			..Default::default()
		})?;
		Ok(())
	}

	pub fn update_media_volume(&mut self) -> Result<()> {
		self.media_controls.set_volume(self.config.volume as f64)?;
		Ok(())
	}

	pub fn update_media_progress(&mut self) -> Result<()> {
		let progress = self.playlist.progress;
		let playback = if self.playlist.remaining.is_empty() {
			MediaPlayback::Stopped
		} else if self.player.is_paused() {
			MediaPlayback::Paused {
				progress: Some(MediaPosition(progress)),
			}
		} else {
			MediaPlayback::Playing {
				progress: Some(MediaPosition(progress)),
			}
		};
		self.media_controls.set_playback(playback)?;
		Ok(())
	}

	pub fn get_current_duration(&self) -> Option<Duration> {
		let first = self.playlist.remaining.front()?;
		let song = self.files.get(first)?;
		song.duration
	}
}

fn setup_media_controls(cmd_sender: Sender<String>) -> Result<MediaControls> {
	#[cfg(not(target_os = "windows"))]
	let hwnd = None;
	#[cfg(target_os = "windows")]
	let hwnd = todo!();

	let config = PlatformConfig {
		dbus_name: "music.player.ttmp",
		display_name: "ttmp",
		hwnd,
	};

	let mut controls = MediaControls::new(config)?;
	controls.attach(move |event| {
		let cmd = match event {
			MediaControlEvent::Play => Borrowed("p+"),
			MediaControlEvent::Pause => Borrowed("p-"),
			MediaControlEvent::Toggle => Borrowed("p"),
			MediaControlEvent::Next => Borrowed("pn"),
			MediaControlEvent::Previous => Borrowed("pp"),
			MediaControlEvent::Stop => return, // not sure what im supposed to do here
			MediaControlEvent::Seek(dir) => Borrowed(match dir {
				SeekDirection::Forward => "gf 5s",
				SeekDirection::Backward => "gb 5s",
			}),
			MediaControlEvent::SeekBy(dir, amount) => Owned(match dir {
				SeekDirection::Forward => format!("gf {}s", amount.as_secs_f32()),
				SeekDirection::Backward => format!("gb {}s", amount.as_secs_f32()),
			}),
			MediaControlEvent::SetPosition(pos) => Owned(format!("g {}s", pos.0.as_secs_f32())),
			MediaControlEvent::SetVolume(vol) => Owned(format!("pv {vol}")),
			MediaControlEvent::OpenUri(_uri) => {
				eprintln!("the OpenUri media control is currently not supported"); // TODO
				return;
			}
			MediaControlEvent::Raise => return, // ttmp doesn't control the terminal window
			MediaControlEvent::Quit => Borrowed("q"),
		};
		if let Err(e) = cmd_sender.send(cmd.into_owned()) {
			eprintln!("error while sending command from media control event: {e}");
		}
	})?;
	Ok(controls)
}
