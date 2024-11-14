use super::get_savedata_path;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, result, time::Duration};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Playlist {
	/// how far you are into the current song
	pub progress: Duration,
	/// the remaining songs in order (current song included)
	pub remaining: Vec<PathBuf>,
}

impl Playlist {
	pub fn load(name: &str) -> Result<Self> {
		let path = get_savedata_path().join(format!("list/{}.ron", name));
		let config_string = fs::read_to_string(path)?;
		let config = ron::from_str(&config_string).map_err(Box::new)?;
		Ok(config)
	}

	pub fn save(&self, name: &str) -> Result<()> {
		let mut pretty_config = PrettyConfig::new();
		pretty_config.indentor = "\t".to_owned();
		pretty_config.new_line = "\n".to_owned();

		let config_string = ron::ser::to_string_pretty(self, pretty_config).map_err(Box::new)?;
		let path = get_savedata_path().join(format!("list/{}.ron", name));
		fs::write(path, config_string)?;
		Ok(())
	}

	pub fn remove(name: &str) -> Result<()> {
		let path = get_savedata_path().join(format!("list/{}.ron", name));
		fs::remove_file(path)?;
		Ok(())
	}
}

type Result<T> = result::Result<T, PlaylistError>;

#[derive(Error, Debug)]
pub enum PlaylistError {
	#[error("io error: {0}")]
	Io(#[from] io::Error),

	// these are wrapped in Box, because SpannedError is 88 bytes and Error is 72 bytes
	#[error("ron spanned error: {0}")]
	RonSpanned(#[from] Box<ron::error::SpannedError>),
	#[error("ron error: {0}")]
	Ron(#[from] Box<ron::Error>),
}
