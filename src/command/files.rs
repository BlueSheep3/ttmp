use super::{
	error::{CommandError::NoFilePlaying, Result},
	play::next_song,
	CommandReturn,
};
use crate::{
	command::files::fs::DirEntry,
	data::{config::Config, context::Context},
};
use std::{
	fs,
	path::{Path, PathBuf},
};

pub fn reload_files(config: &mut Config) -> Result<()> {
	config.reload_files()?;
	Ok(())
}

pub fn show_full_path(ctx: &Context) -> Result<()> {
	let current = ctx.playlist.remaining.first().ok_or(NoFilePlaying)?;
	if current.is_absolute() {
		println!("{}", current.display());
	} else {
		println!("{}", ctx.config.path.join(current).display());
	}
	Ok(())
}

pub fn delete_current(ctx: &mut Context) -> Result<CommandReturn> {
	let current = ctx.playlist.remaining.first().ok_or(NoFilePlaying)?;
	ctx.config.files.remove(current);
	fs::remove_file(ctx.config.path.join(current))?;
	println!("File deleted successfully.");
	Ok(next_song(ctx))
}

pub fn move_file(ctx: &mut Context, destination_folder: &[&str]) -> Result<()> {
	let input = destination_folder.join(" ");
	let destination_folder = Path::new(&input);
	let file_name = ctx.playlist.remaining.first_mut().ok_or(NoFilePlaying)?;
	let song_name = file_name
		.file_name()
		.expect("Failed to get file name from the path.")
		.to_string_lossy()
		.to_string();
	let destination_full = ctx.config.path.join(destination_folder).join(&song_name);

	let new_folder = fs::metadata(&destination_full).is_err();
	fs::rename(ctx.config.path.join(&file_name), &destination_full)?;

	let destination = destination_folder.join(&song_name);
	*file_name = destination.clone();
	let current = &destination;
	if let Some(file_data) = ctx.config.files.remove(current) {
		ctx.config.files.insert(destination, file_data);
	}
	if new_folder {
		println!("Succesfully moved File");
	} else {
		println!(
			"Created new Folder to move file to {}",
			destination_full.to_string_lossy()
		);
	}
	Ok(())
}

pub fn show_directories(config: &Config) -> Result<()> {
	if let Some(folder_name) = &config.path.file_name() {
		println!("{}", folder_name.to_string_lossy());
	}
	folders_recursive(&config.path, "", false, &mut 21)
}

fn folders_recursive(path: &PathBuf, layers: &str, is_ending: bool, max: &mut i32) -> Result<()> {
	let entries = fs::read_dir(path)?;
	let subdirs: Vec<DirEntry> = entries
		.filter_map(|res| res.ok())
		.filter(|entry| entry.path().is_dir())
		.collect();

	let count = subdirs.len();
	for (index, entry) in subdirs.into_iter().enumerate() {
		if *max == 0 {
			return Ok(());
		}
		*max -= 1;
		let is_last = index == count - 1;
		let new_layer = if is_last { "└── " } else { "├── " };
		if let Some(folder_name) = entry.path().file_name() {
			println!(
				"{}{}",
				layers.to_owned() + new_layer,
				folder_name.to_string_lossy()
			);
		}
		let is_ending = is_last || is_ending;
		let new_layer = if is_ending { "    " } else { "│   " };
		let new_layers = layers.to_owned() + new_layer;
		let subpath = entry.path();
		folders_recursive(&subpath, &new_layers, is_ending, max)?;
	}
	Ok(())
}
