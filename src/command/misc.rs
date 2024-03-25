use super::{
	error::{CommandError::NoFilePlaying, Result},
	play::{self, next_song},
};
use crate::config::Config;
use rodio::Sink;
use std::{fs, path::Path, time::Duration};

pub fn delete_current(config: &mut Config, sink: &Sink) -> Result<()> {
	let current = config.remaining.first().ok_or(NoFilePlaying)?;
	config.files.remove(current);
	fs::remove_file(config.parent_path.join(current))?;
	println!("File deleted successfully.");
	next_song(sink);
	Ok(())
}

pub fn move_file(config: &mut Config, destination_folder: &[&str]) {
	let input = destination_folder.join(" ");
	let destination_folder = Path::new(&input);
	let file_name = config.remaining.first();
	let file_name = match file_name {
		Some(name) => name,
		None => {
			println!("tried to move file, but no file is playing");
			return;
		}
	};
	let song_name = file_name
		.file_name()
		.expect("Failed to get file name from the path.")
		.to_string_lossy()
		.to_string();
	let destination_full = config
		.parent_path
		.join(destination_folder)
		.join(song_name.clone());

	if let Err(err) = fs::rename(config.parent_path.join(file_name), destination_full.clone()) {
		println!("Error moving file: {}", err);
	} else {
		let destination = destination_folder.join(song_name.clone());
		config.remaining[0] = destination.clone();
		let current = &destination;
		if let Some(file_data) = config.files.remove(current) {
			config.files.insert(destination, file_data);
		}
	}
}

pub fn show_full_path(config: &Config) -> Result<()> {
	let current = config.remaining.first().ok_or(NoFilePlaying)?;
	if current.is_absolute() {
		println!("{}", current.display());
	} else {
		println!("{}", config.parent_path.join(current).display());
	}
	Ok(())
}

pub fn reload_files(config: &mut Config) -> Result<()> {
	config.reload_files()?;
	Ok(())
}

pub fn enforce_max(config: &mut Config, max: &str) -> Result<()> {
	let max = max.parse::<usize>()?;
	config.remaining.truncate(max);
	Ok(())
}

pub fn reset_remaining(config: &mut Config, sink: &Sink) {
	config.remaining = config.files.keys().cloned().collect();
	config.looping_songs.clear();
	config.current_progress = Duration::ZERO;
	next_song(sink);
	if config.start_playing_immediately {
		play::start_playing(sink)
	}
}
