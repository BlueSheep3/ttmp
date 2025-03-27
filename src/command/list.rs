//! commands that operate between different playlists,
//! for example: loading, saving, switching lists,
//! but not: playing, pausing, skipping songs

use super::{
	error::{
		CommandError::{
			DeleteCurrentPlaylist, ListNameBadChar, ListNameEmpty, SaveOverCurrentPlaylist,
		},
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

/// Forces the name of a playlist to be in a nice format.\
/// If the bad format is simple, like being uppercase, it will simply be corrected,
/// but if anything unrecoverable is detected in the name, an error will be returned.
fn force_str_format(name: &str) -> Result<String> {
	if name.is_empty() {
		return Err(ListNameEmpty);
	}
	if let Some(c) = name
		.chars()
		.find(|x| matches!(x, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
	{
		return Err(ListNameBadChar(c));
	}
	Ok(name.to_lowercase())
}

pub fn new_empty(ctx: &Context, name: &str) -> Result<()> {
	let name = force_str_format(name)?;
	if name == ctx.config.current_playlist {
		return Err(SaveOverCurrentPlaylist);
	}
	let list = Playlist::default();
	list.save(&name)?;
	Ok(())
}

pub fn duplicate_to(ctx: &Context, name: &str) -> Result<()> {
	let name = force_str_format(name)?;
	if name == ctx.config.current_playlist {
		return Err(SaveOverCurrentPlaylist);
	}
	ctx.playlist.save(&name)?;
	Ok(())
}

pub fn append_from(ctx: &mut Context, name: &str) -> Result<CommandReturn> {
	let name = force_str_format(name)?;
	// the current playlist is saved lazily, meaning Playlist::load(name)
	// would get a previous version of ctx.playlist
	let songs = if name == ctx.config.current_playlist {
		ctx.playlist.remaining.clone()
	} else {
		Playlist::load(&name)?.remaining
	};
	let was_empty = ctx.playlist.remaining.is_empty();
	ctx.playlist.remaining.extend_from_slice(&songs);

	Ok(if was_empty {
		CommandReturn::ReloadFirstSong
	} else {
		CommandReturn::Nothing
	})
}

pub fn copy_from(ctx: &mut Context, name: &str) -> Result<CommandReturn> {
	let name = force_str_format(name)?;
	// the current playlist is saved lazily, meaning Playlist::load(name)
	// would get a previous version of ctx.playlist
	if name == ctx.config.current_playlist {
		return Ok(CommandReturn::Nothing);
	}
	ctx.playlist = Playlist::load(&name)?;
	Ok(CommandReturn::ReloadFirstSong)
}

pub fn remove(ctx: &Context, name: &str) -> Result<()> {
	let name = force_str_format(name)?;
	if name == ctx.config.current_playlist {
		return Err(DeleteCurrentPlaylist);
	}
	Playlist::remove(&name)?;
	Ok(())
}

pub fn switch_to(ctx: &mut Context, name: &str) -> Result<CommandReturn> {
	let name = force_str_format(name)?;
	if ctx.program_mode.can_save() {
		ctx.playlist.save(&ctx.config.current_playlist)?;
	}
	ctx.playlist = Playlist::load(&name)?;
	ctx.config.current_playlist = name;
	Ok(CommandReturn::ReloadFirstSong)
}
