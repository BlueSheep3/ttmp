//! commands that operate between different playlists,
//! for example: loading, saving, switching lists,
//! but not: playing, pausing, skipping songs

use super::{
	error::{
		CommandError::{DeleteCurrentPlaylist, SaveOverCurrentPlaylist},
		Result,
	},
	CommandReturn,
};
use crate::data::{context::Context, playlist::Playlist};

pub fn get_list_names(ctx: &Context) -> Result<()> {
	println!("All Playlist Names:");
	for name in Playlist::get_all_names()? {
		let first_char = if name == ctx.config.current_playlist {
			'>'
		} else {
			'-'
		};
		println!("{} {}", first_char, name);
	}
	Ok(())
}

pub fn new_empty(ctx: &Context, name: &str) -> Result<()> {
	if name == ctx.config.current_playlist {
		return Err(SaveOverCurrentPlaylist);
	}
	let list = Playlist::default();
	list.save(name)?;
	Ok(())
}

pub fn duplicate(ctx: &Context, name: &str) -> Result<()> {
	if name == ctx.config.current_playlist {
		return Err(SaveOverCurrentPlaylist);
	}
	ctx.playlist.save(name)?;
	Ok(())
}

pub fn copy_from(ctx: &mut Context, name: &str) -> Result<CommandReturn> {
	ctx.playlist = Playlist::load(name)?;
	Ok(CommandReturn::ReloadFirstSong)
}

pub fn remove(ctx: &Context, name: &str) -> Result<()> {
	if name == ctx.config.current_playlist {
		return Err(DeleteCurrentPlaylist);
	}
	Playlist::remove(name)?;
	Ok(())
}

pub fn switch_to(ctx: &mut Context, name: &str) -> Result<CommandReturn> {
	ctx.playlist.save(&ctx.config.current_playlist)?;
	ctx.playlist = Playlist::load(name)?;
	ctx.config.current_playlist = name.to_owned();
	Ok(CommandReturn::ReloadFirstSong)
}
