use super::{
	CommandReturn,
	error::{
		CommandError::{InvalidFileName, NoFilePlaying},
		Result,
	},
	play::next_song,
};
use crate::data::{context::Context, files::Files};
use std::{fs, path::Path};

pub fn reload_files(files: &mut Files) -> Result<()> {
	files.reload_files()?;
	Ok(())
}

pub fn show_full_path(ctx: &mut Context) -> Result<()> {
	let current = ctx.playlist.remaining.first().ok_or(NoFilePlaying)?;
	if current.is_absolute() {
		ctx.cmd_out += &current.display().to_string();
		ctx.cmd_out.push('\n');
	} else {
		ctx.cmd_out += &ctx.files.root.join(current).display().to_string();
		ctx.cmd_out.push('\n');
	}
	Ok(())
}

pub fn delete_current(ctx: &mut Context) -> Result<CommandReturn> {
	let current = ctx.playlist.remaining.first().ok_or(NoFilePlaying)?;
	ctx.files.remove(current);
	fs::remove_file(ctx.files.root.join(current))?;
	ctx.cmd_out += "File deleted successfully.\n";
	Ok(next_song(ctx))
}

pub fn move_file(ctx: &mut Context, destination_folder: &[&str]) -> Result<()> {
	let input = destination_folder.join(" ");
	let destination_folder = Path::new(&input);
	let file_name = ctx.playlist.remaining.first_mut().ok_or(NoFilePlaying)?;
	let song_name = file_name
		.file_name()
		.ok_or(InvalidFileName(file_name.clone()))?
		.to_string_lossy()
		.to_string();
	let destination_full = ctx.files.root.join(destination_folder).join(&song_name);

	let new_folder = fs::metadata(&destination_full).is_err();
	fs::rename(ctx.files.root.join(&file_name), &destination_full)?;

	let destination = destination_folder.join(&song_name);
	*file_name = destination.clone();
	let current = &destination;
	if let Some(file_data) = ctx.files.remove(current) {
		ctx.files.insert(destination, file_data);
	}
	if new_folder {
		ctx.cmd_out += "Succesfully moved File\n";
	} else {
		ctx.cmd_out += &format!(
			"Created new Folder to move file to {}\n",
			destination_full.to_string_lossy()
		);
	}
	Ok(())
}

pub fn show_directories(files: &Files, cmd_out: &mut String) -> Result<()> {
	if let Some(folder_name) = &files.root.file_name() {
		*cmd_out += &folder_name.to_string_lossy();
		cmd_out.push('\n');
	}
	folders_recursive(cmd_out, &files.root, "", false, &mut 21)
}

fn folders_recursive(
	cmd_out: &mut String,
	path: &Path,
	layers: &str,
	is_ending: bool,
	max: &mut i32,
) -> Result<()> {
	let entries = fs::read_dir(path)?;
	let mut subdirs = entries
		.filter_map(|res| res.ok())
		.filter(|entry| entry.path().is_dir())
		.peekable();

	while let Some(entry) = subdirs.next() {
		if *max == 0 {
			return Ok(());
		}
		*max -= 1;
		let is_last = subdirs.peek().is_none();
		let new_layer = if is_last { "└── " } else { "├── " };
		if let Some(folder_name) = entry.path().file_name() {
			*cmd_out += &format!(
				"{}{}\n",
				layers.to_owned() + new_layer,
				folder_name.to_string_lossy()
			);
		}
		let is_ending = is_last || is_ending;
		let new_layer = if is_ending { "    " } else { "│   " };
		let new_layers = layers.to_owned() + new_layer;
		let subpath = entry.path();
		folders_recursive(cmd_out, &subpath, &new_layers, is_ending, max)?;
	}
	Ok(())
}
