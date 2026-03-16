//! commands that operate between different playlists,
//! for example: loading, saving, switching lists,
//! but not: playing, pausing, skipping songs

use super::{
	CommandReturn,
	error::{
		CommandError::{
			DeleteCurrentPlaylist, ListNameBadChar, ListNameEmpty, SaveOverCurrentPlaylist,
		},
		Result,
	},
};
use crate::data::{context::Context, playlist::Playlist};

pub fn get_list_names(ctx: &mut Context) -> Result<()> {
	ctx.cmd_out += "All Playlist Names:\n";
	for name in Playlist::get_all_names(&ctx.savedata_path)? {
		let first_char = if name == ctx.config.current_playlist {
			'>'
		} else {
			'-'
		};
		ctx.cmd_out += &format!("{first_char} {name}\n");
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
	list.save(&name, &ctx.savedata_path)?;
	Ok(())
}

pub fn duplicate_to(ctx: &Context, name: &str) -> Result<()> {
	let name = force_str_format(name)?;
	if name == ctx.config.current_playlist {
		return Err(SaveOverCurrentPlaylist);
	}
	ctx.playlist.save(&name, &ctx.savedata_path)?;
	Ok(())
}

pub fn append_from(ctx: &mut Context, name: &str) -> Result<CommandReturn> {
	let name = force_str_format(name)?;
	// the current playlist is saved lazily, meaning Playlist::load(name)
	// would get a previous version of ctx.playlist
	let songs = if name == ctx.config.current_playlist {
		ctx.playlist.remaining.clone()
	} else {
		Playlist::load(&name, &ctx.savedata_path)?.remaining
	};
	let was_empty = ctx.playlist.remaining.is_empty();
	ctx.playlist.remaining.extend(songs);

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
	ctx.playlist = Playlist::load(&name, &ctx.savedata_path)?;
	Ok(CommandReturn::ReloadFirstSong)
}

pub fn remove(ctx: &Context, name: &str) -> Result<()> {
	let name = force_str_format(name)?;
	if name == ctx.config.current_playlist {
		return Err(DeleteCurrentPlaylist);
	}
	Playlist::remove(&name, &ctx.savedata_path)?;
	Ok(())
}

pub fn switch_to(ctx: &mut Context, name: &str) -> Result<CommandReturn> {
	let name = force_str_format(name)?;
	if ctx.program_mode.can_save() {
		ctx.playlist
			.save(&ctx.config.current_playlist, &ctx.savedata_path)?;
	}
	ctx.playlist = Playlist::load(&name, &ctx.savedata_path)?;
	ctx.config.current_playlist = name;
	Ok(CommandReturn::ReloadFirstSong)
}
