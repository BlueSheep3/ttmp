use super::play::{self, next_song};
use crate::config::Config;
use rodio::Sink;
use std::{fs, path::Path, time::Duration};

pub fn delete_current(config: &mut Config, sink: &Sink) {
	let Some(current) = config.remaining.first() else {
		println!("No File currently playing");
		return;
	};
	config.files.remove(current);
	if let Err(err) = fs::remove_file(config.parent_path.join(current)) {
		println!("Error deleting file: {}", err);
	} else {
		println!("File deleted successfully.");
	}
	next_song(sink);
}

pub fn move_file(config: &mut Config, destination_folder: &[&str]) {
	let input = destination_folder.join(" ");
	let destination_folder = Path::new(&input);
	let file_name = config
		.remaining
		.first()
		.expect("No file currently playing.");
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
		let current = &config.remaining[0];
		if let Some(file_data) = config.files.remove(current) {
			config.files.insert(destination, file_data);
		}
	}
}

pub fn show_full_path(config: &Config) {
	let Some(current) = config.remaining.first() else {
		println!("No File currently playing");
		return;
	};
	if current.is_absolute() {
		println!("{}", current.display());
	} else {
		println!("{}", config.parent_path.join(current).display());
	}
}

pub fn reload_files(config: &mut Config) {
	config.reload_files().unwrap_or_else(|e| {
		println!("failed to add new files: {}", e);
	});
}

pub fn show_progress(config: &Config) {
	println!("Progress: {:.02}", config.current_progress.as_secs_f32());
}

pub fn enforce_max(config: &mut Config, max: &str) {
	if let Ok(m) = max.parse::<usize>() {
		config.remaining.truncate(m);
	} else {
		println!("Invalid max: {}", max);
	}
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
