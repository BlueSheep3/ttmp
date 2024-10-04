//! commands that operate between different playlists,
//! for example: loading, saving, switching lists,
//! but not: playing, pausing, skipping songs

use super::{
	error::{
		CommandError::{DeleteCurrentPlaylist, SaveOverCurrentPlaylist},
		Result,
	},
	play,
};
use crate::data::{context::Context, playlist::Playlist};

pub fn get_current_name(ctx: &Context) {
	println!("Current Playlist: {}", ctx.config.current_playlist);
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

// FIXME loading in new songs like this has weird behaviour
pub fn copy_from(ctx: &mut Context, name: &str) -> Result<()> {
	ctx.playlist = Playlist::load(name)?;
	play::reload_current_song(ctx);
	Ok(())
}

pub fn remove(ctx: &Context, name: &str) -> Result<()> {
	if name == ctx.config.current_playlist {
		return Err(DeleteCurrentPlaylist);
	}
	Playlist::remove(name)?;
	Ok(())
}

// FIXME loading in new songs like this has weird behaviour
pub fn switch_to(ctx: &mut Context, name: &str) -> Result<()> {
	ctx.playlist.save(&ctx.config.current_playlist)?;
	ctx.playlist = Playlist::load(name)?;
	ctx.config.current_playlist = name.to_owned();
	play::reload_current_song(ctx);
	Ok(())
}
