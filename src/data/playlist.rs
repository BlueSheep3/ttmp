use super::error::{DataError, Result};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
	borrow::Cow,
	fs,
	path::{Path, PathBuf},
	time::Duration,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Playlist {
	/// how far you are into the current song
	pub progress: Duration,
	/// the remaining songs in order (current song included)
	pub remaining: Vec<PathBuf>,
	/// the songs you have already listened to in order
	#[serde(default)]
	pub previous: Vec<PathBuf>,
}

impl Playlist {
	pub fn load(name: &str, savedata_path: &Path) -> Result<Self> {
		let path = savedata_path.join(format!("list/{name}.ron"));
		let config_string = fs::read_to_string(path)?;
		let config = ron::from_str(&config_string).map_err(Box::new)?;
		Ok(config)
	}

	pub fn save(&self, name: &str, savedata_path: &Path) -> Result<()> {
		let mut pretty_config = PrettyConfig::new();
		pretty_config.indentor = Cow::Borrowed("\t");
		pretty_config.new_line = Cow::Borrowed("\n");

		let config_string = ron::ser::to_string_pretty(self, pretty_config).map_err(Box::new)?;
		let path = savedata_path.join(format!("list/{name}.ron"));
		fs::write(path, config_string)?;
		Ok(())
	}

	pub fn remove(name: &str, savedata_path: &Path) -> Result<()> {
		let path = savedata_path.join(format!("list/{name}.ron"));
		fs::remove_file(path)?;
		Ok(())
	}

	/// Gets the names of all playlists in the `list` folder of the appdata,
	/// in such a way that its usable in the [`Playlist::load`] function.
	pub fn get_all_names(savedata_path: &Path) -> Result<Vec<String>> {
		let mut names = Vec::new();
		for list in fs::read_dir(savedata_path.join("list"))? {
			let name = list?
				.file_name()
				.into_string()
				.map_err(DataError::FileNotUtf8Name)?;
			let base = name.strip_suffix(".ron").unwrap_or(&name);
			names.push(base.to_owned());
		}
		Ok(names)
	}

	pub fn next_song(&mut self) {
		let removed = self.remaining.remove(0);
		self.previous.push(removed);
	}

	pub fn previous_song(&mut self) {
		let removed = self.previous.pop().unwrap();
		self.remaining.insert(0, removed);
	}
}
