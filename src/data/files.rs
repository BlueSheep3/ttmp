use super::error::Result;
use crate::serializer;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
	borrow::Cow,
	collections::{HashMap, HashSet},
	ffi::OsStr,
	fs, io,
	ops::{Deref, DerefMut},
	path::{Path, PathBuf},
	process::{Command, Stdio},
	result,
	time::Duration,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Files {
	/// the folder that all the files are in.
	pub root: PathBuf,
	/// all music files, paths should be relative to `root`.
	#[serde(serialize_with = "serializer::sorted_hashmap")]
	pub mappings: HashMap<PathBuf, FileData>,
}

impl Files {
	pub fn empty_with_root(root: PathBuf) -> Self {
		Self {
			root,
			mappings: HashMap::new(),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FileData {
	#[serde(serialize_with = "serializer::sorted_hashset")]
	pub tags: HashSet<String>,
	/// a cache of how long the song is.
	/// this gets updated when a song finishes playing.
	pub duration: Option<Duration>,
}

impl Deref for Files {
	type Target = HashMap<PathBuf, FileData>;

	fn deref(&self) -> &Self::Target {
		&self.mappings
	}
}

impl DerefMut for Files {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.mappings
	}
}

impl Files {
	pub fn load(savedata_path: &Path) -> Result<Self> {
		let path = savedata_path.join("files.ron");
		let files_string = fs::read_to_string(path)?;
		let files = ron::from_str(&files_string).map_err(Box::new)?;
		Ok(files)
	}

	pub fn save(&self, savedata_path: &Path) -> Result<()> {
		let mut pretty_config = PrettyConfig::new();
		pretty_config.indentor = Cow::Borrowed("\t");
		pretty_config.new_line = Cow::Borrowed("\n");

		let files_string = ron::ser::to_string_pretty(self, pretty_config).map_err(Box::new)?;
		let path = savedata_path.join("files.ron");
		fs::write(path, files_string)?;
		Ok(())
	}

	/// add all files that are in the system, but not in the config,
	/// and remove all files that are in the config but not in the system.
	pub fn reload_files(&mut self) -> Result<()> {
		let system_files = get_all_files_in(&self.root)?;

		for full_path in &system_files {
			let rel_path = full_path
				.strip_prefix(&self.root)
				.unwrap_or_else(|_| unreachable!())
				.to_path_buf();
			self.mappings.entry(rel_path).or_default();
		}

		let mut files_to_remove = Vec::new();

		for rel_path in self.mappings.keys() {
			let full_path = self.root.join(rel_path);
			if !system_files.contains(&full_path) {
				files_to_remove.push(rel_path.clone());
			}
		}
		for rel_path in files_to_remove {
			self.mappings.remove(&rel_path);
		}

		Ok(())
	}
}

/// gets all files in a folder, including subfolders
fn get_all_files_in(path: &Path) -> result::Result<Vec<PathBuf>, io::Error> {
	let mut files = vec![];

	for entry in fs::read_dir(path)? {
		let entry = entry?;
		if entry.file_type()?.is_dir() {
			files.extend(get_all_files_in(&entry.path())?);
		} else {
			let name = entry.file_name();
			let is_music = name.to_str().map(is_music_file).unwrap_or(false);
			if is_music {
				files.push(entry.path());
			}
		}
	}
	Ok(files)
}

pub fn make_temp_mp4_copy(absolute_song_path: &Path, savedata_path: &Path) -> Result<PathBuf> {
	let output_path = savedata_path.join("tempmp4.mp3");
	if output_path.exists() {
		fs::remove_file(&output_path)?;
	}
	Command::new("ffmpeg")
		.args([
			OsStr::new("-i"),
			absolute_song_path.as_ref(),
			"-vn".as_ref(),
			"-acodec".as_ref(),
			"libmp3lame".as_ref(),
			output_path.as_ref(),
		])
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.status()?;
	Ok(output_path)
}

pub fn is_mp4_file(file_name: &str) -> bool {
	file_name.ends_with(".mp4")
}

fn is_music_file(file_name: &str) -> bool {
	[".mp3", ".wav", ".ogg", ".mp4"]
		.into_iter()
		.any(|end| file_name.ends_with(end))
}
