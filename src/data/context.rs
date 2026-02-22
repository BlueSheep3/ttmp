use super::{
	config::{Config, StartPlayState},
	error::Result,
	files::{FileData, Files},
	playlist::Playlist,
};
use rodio::{OutputStream, OutputStreamBuilder, Sink};
use std::{env::ArgsOs, ffi::OsString, time::Duration};

pub struct Context {
	pub program_mode: ProgramMode,
	pub cmd_out: String,
	pub config: Config,
	pub files: Files,
	pub playlist: Playlist,
	pub sink: Sink,

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
	/// autoamitcally figures out which mode to initialize the context in
	pub fn new_automatic(args: ArgsOs) -> Result<Self> {
		let args = args.collect::<Vec<OsString>>();
		if let [_, files @ ..] = args.as_slice()
			&& !files.is_empty()
		{
			Self::new_temp(files)
		} else {
			Self::new_main()
		}
	}

	pub fn new_main() -> Result<Self> {
		let program_mode = ProgramMode::Main;
		let config = Config::load()?;
		let files = Files::load()?;
		let playlist = Playlist::load(&config.current_playlist)?;
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
			_stream_handle: stream_handle,
		};
		ctx.init_sink();
		Ok(ctx)
	}

	pub fn new_temp(file_paths: &[OsString]) -> Result<Self> {
		let program_mode = ProgramMode::Temp;
		let mut config = Config::load()?;
		let mut files = Files::load()?;
		let mut playlist = Playlist::default();
		let mut stream_handle = OutputStreamBuilder::open_default_stream()?;
		stream_handle.log_on_drop(false);
		let sink = Sink::connect_new(stream_handle.mixer());

		config.start_play_state = StartPlayState::Always;
		config.current_playlist = "temp".to_owned();
		files.mappings = file_paths
			.iter()
			.map(|f| (f.into(), FileData::default()))
			.collect();
		playlist.remaining = file_paths.iter().map(From::from).collect();
		playlist.progress = Duration::ZERO;

		let ctx = Self {
			program_mode,
			cmd_out: String::new(),
			config,
			files,
			playlist,
			sink,
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
