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
		let path = get_savedata_path().join(format!("list/{name}.ron"));
		let config_string = fs::read_to_string(path)?;
		let config = ron::from_str(&config_string).map_err(Box::new)?;
		Ok(config)
	}

	pub fn save(&self, name: &str) -> Result<()> {
		let mut pretty_config = PrettyConfig::new();
		pretty_config.indentor = "\t".to_owned();
		pretty_config.new_line = "\n".to_owned();

		let config_string = ron::ser::to_string_pretty(self, pretty_config).map_err(Box::new)?;
		let path = get_savedata_path().join(format!("list/{name}.ron"));
		fs::write(path, config_string)?;
		Ok(())
	}

	pub fn remove(name: &str) -> Result<()> {
		let path = get_savedata_path().join(format!("list/{name}.ron"));
		fs::remove_file(path)?;
		Ok(())
	}

	/// Gets the names of all playlists in the `list` folder of the appdata,
	/// in such a way that its usable in the [`Playlist::load`] function.
	pub fn get_all_names() -> Result<Vec<String>> {
		let mut names = Vec::new();
		for list in fs::read_dir(get_savedata_path().join("list"))? {
			let name = list?
				.file_name()
				.into_string()
				.map_err(PlaylistError::FileNotUtf8Name)?;
			let base = name.strip_suffix(".ron").unwrap_or(&name);
			names.push(base.to_owned());
		}
		Ok(names)
	}
}

type Result<T> = result::Result<T, PlaylistError>;

#[derive(Error, Debug)]
pub enum PlaylistError {
	#[error("io error: {0}")]
	Io(#[from] io::Error),
	#[error("the file name {0:?} is not valid utf8")]
	FileNotUtf8Name(std::ffi::OsString),

	// these are wrapped in Box, because SpannedError is 88 bytes and Error is 72 bytes
	#[error("ron spanned error: {0}")]
	RonSpanned(#[from] Box<ron::error::SpannedError>),
	#[error("ron error: {0}")]
	Ron(#[from] Box<ron::Error>),
}
