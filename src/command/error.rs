use crate::{
	data::{config, files, playlist},
	duration,
};
use std::{io, path::PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CommandError>;

#[derive(Error, Debug)]
pub enum CommandError {
	#[error("config error: {0}")]
	Config(#[from] config::ConfigError),
	#[error("files error: {0}")]
	Files(#[from] files::FilesError),
	#[error("playlist error: {0}")]
	Playlist(#[from] playlist::PlaylistError),
	#[error("io error: {0}")]
	Io(#[from] io::Error),
	#[error("Failed while parsing Integer: {0}")]
	ParseInt(#[from] std::num::ParseIntError),
	#[error("Failed while parsing Float: {0}")]
	ParseFloat(#[from] std::num::ParseFloatError),
	#[error("Failed while parsing Duration: {0}")]
	DurationParse(#[from] duration::DurationParseError),

	#[error("Uknown or Invalid command: {0}")]
	UknownOrInvalidCommand(String),
	#[error("No help available for: {0}")]
	NoHelpAvailable(String),
	#[error("No File currently playing")]
	NoFilePlaying,
	#[error("Song not in Files list")]
	NotInFiles(PathBuf),
	#[error("Macro already exists: {0}")]
	MacroAlreadyExists(String),
	#[error("Macro does not exist: {0}")]
	MacroDoesNotExist(String),
	#[error("Volume can't be less than 0%, but got: {0}")]
	VolumeTooLow(f32),
	#[error("Volume can't be more than 300%, to protect your ears, but got: {0}")]
	VolumeTooHigh(f32),
	#[error("Can't save in this mode")]
	SaveInWrongMode,
	#[error("Can't save over the current playlist")]
	SaveOverCurrentPlaylist,
	#[error("Can't delete the current playlist")]
	DeleteCurrentPlaylist,
}
