use super::{
	config::{Config, ConfigError, FileData, StartPlayState},
	playlist::{Playlist, PlaylistError},
};
use rodio::{OutputStream, OutputStreamHandle, PlayError, Sink, StreamError};
use std::{collections::HashMap, path::Path, result, time::Duration};
use thiserror::Error;

pub struct Context {
	pub program_mode: ProgramMode,
	pub config: Config,
	pub playlist: Playlist,
	pub sink: Sink,

	// these are just here, so the music doesnt stop, due to them being dropped
	stream: OutputStream,
	stream_handle: OutputStreamHandle,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProgramMode {
	/// The mode used when running the program normally.
	Main,
	/// The mode used when opening individual music files.
	/// In this mode, nothing is saved when quitting normally.
	Temp,
}

impl Context {
	pub fn new_main() -> Result<Self> {
		let program_mode = ProgramMode::Main;
		let config = Config::load()?;
		let playlist = Playlist::load(&config.main_playlist)?;
		let (stream, stream_handle) = OutputStream::try_default()?;
		let sink = Sink::try_new(&stream_handle)?;

		let ctx = Self {
			program_mode,
			config,
			playlist,
			sink,
			stream,
			stream_handle,
		};
		ctx.init_sink();
		Ok(ctx)
	}

	pub fn new_temp(file: &Path) -> Result<Self> {
		let program_mode = ProgramMode::Temp;
		let mut config = Config::load()?;
		let mut playlist = Playlist::default();
		let (stream, stream_handle) = OutputStream::try_default()?;
		let sink = Sink::try_new(&stream_handle)?;

		config.path = file.to_owned();
		config.files = HashMap::from([(file.to_owned(), FileData::default())]);
		config.start_play_state = StartPlayState::Always;
		playlist.remaining = vec![file.to_owned()];
		playlist.progress = Duration::ZERO;
		playlist.dont_save_at = Duration::ZERO;

		let ctx = Self {
			program_mode,
			config,
			playlist,
			sink,
			stream,
			stream_handle,
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
		let first = self.playlist.remaining.first()?;
		let song = self.config.files.get(first)?;
		song.duration
	}
}

type Result<T> = result::Result<T, ContextError>;

#[derive(Error, Debug)]
pub enum ContextError {
	#[error("{0}")]
	ConfigError(#[from] ConfigError),
	#[error("{0}")]
	PlaylistError(#[from] PlaylistError),
	#[error("{0}")]
	StreamError(#[from] StreamError),
	#[error("{0}")]
	PlayError(#[from] PlayError),
}
