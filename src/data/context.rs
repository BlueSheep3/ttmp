use super::{
	config::{Config, ConfigError, StartPlayState},
	files::{FileData, Files, FilesError},
	playlist::{Playlist, PlaylistError},
};
use rodio::{OutputStream, OutputStreamHandle, PlayError, Sink, StreamError};
use std::{collections::HashMap, path::Path, result, time::Duration};
use thiserror::Error;

pub struct Context {
	pub program_mode: ProgramMode,
	pub config: Config,
	pub files: Files,
	pub playlist: Playlist,
	pub sink: Sink,

	// these are just here, so the music doesnt stop, due to them being dropped
	_stream: OutputStream,
	_stream_handle: OutputStreamHandle,
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
	pub fn new_main() -> Result<Self> {
		let program_mode = ProgramMode::Main;
		let config = Config::load()?;
		let files = Files::load()?;
		let playlist = Playlist::load(&config.current_playlist)?;
		let (stream, stream_handle) = OutputStream::try_default()?;
		let sink = Sink::try_new(&stream_handle)?;

		let ctx = Self {
			program_mode,
			config,
			files,
			playlist,
			sink,
			_stream: stream,
			_stream_handle: stream_handle,
		};
		ctx.init_sink();
		Ok(ctx)
	}

	pub fn new_temp(file: &Path) -> Result<Self> {
		let program_mode = ProgramMode::Temp;
		let mut config = Config::load()?;
		let mut files = Files::load()?;
		let mut playlist = Playlist::default();
		let (stream, stream_handle) = OutputStream::try_default()?;
		let sink = Sink::try_new(&stream_handle)?;

		config.path = file.to_owned();
		config.start_play_state = StartPlayState::Always;
		config.current_playlist = "temp".to_owned();
		files.mappings = HashMap::from([(file.to_owned(), FileData::default())]);
		playlist.remaining = vec![file.to_owned()];
		playlist.progress = Duration::ZERO;

		let ctx = Self {
			program_mode,
			config,
			files,
			playlist,
			sink,
			_stream: stream,
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
		let first = self.playlist.remaining.first()?;
		let song = self.files.get(first)?;
		song.duration
	}
}

type Result<T> = result::Result<T, ContextError>;

#[derive(Error, Debug)]
pub enum ContextError {
	#[error("{0}")]
	Config(#[from] ConfigError),
	#[error("{0}")]
	Files(#[from] FilesError),
	#[error("{0}")]
	Playlist(#[from] PlaylistError),
	#[error("{0}")]
	Stream(#[from] StreamError),
	#[error("{0}")]
	Play(#[from] PlayError),
}
