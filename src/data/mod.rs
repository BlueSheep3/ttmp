pub mod config;
pub mod context;
pub mod files;
pub mod playlist;

use crate::data::{config::Config, files::Files, playlist::Playlist};
use std::{error::Error, fs, path::PathBuf};

pub fn create_default_savedata_if_not_present() -> Result<(), Box<dyn Error>> {
	let path = get_savedata_path().ok_or("cant find savedata path")?;

	if fs::exists(&path)? {
		// already has the savedata, so we do nothing
		return Ok(());
	}

	fs::create_dir_all(path.join("list"))?;

	let music = match dirs::audio_dir() {
		Some(m) => m,
		None => {
			println!(
				"\
Cant find a default Music folder.
Give the path of the folder that contains all your Music.\
"
			);
			loop {
				print!("Music Path: ");
				let music_path = readln();
				let music_path = PathBuf::from(music_path);
				if music_path.is_dir() {
					break music_path;
				}
				println!("The path you provided is not a directory that exists.");
			}
		}
	};

	Config::default().save()?;
	Files::empty_with_root(music).save()?;
	Playlist::default().save("main")?;
	Ok(())
}

fn readln() -> String {
	let mut input = String::new();
	std::io::stdin()
		.read_line(&mut input)
		.expect("reading from stdin failed");

	if input.ends_with('\n') {
		input.pop();
	}
	if input.ends_with('\r') {
		input.pop();
	}

	input
}

fn get_savedata_path() -> Option<PathBuf> {
	let config = dirs::config_dir()?;
	let path = config.join("musicplayer");
	Some(path)
}
