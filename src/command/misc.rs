// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use super::{
	CommandReturn,
	error::{
		CommandError::{NoFilePlaying, SaveInWrongMode},
		Result,
	},
	play,
};
use crate::data::{context::Context, playlist::Playlist};
use std::time::Duration;

pub fn reset_remaining(ctx: &mut Context) -> Result<CommandReturn> {
	ctx.playlist.previous.clear();
	ctx.playlist.remaining = ctx.files.keys().cloned().collect();
	if ctx.config.start_play_state.should_play() {
		play::start_playing(ctx)?;
	}
	Ok(load_in_first_song(ctx))
}

pub fn echo(text: &[&str], cmd_out: &mut String) {
	*cmd_out += &text.join(" ");
	cmd_out.push('\n');
}

pub fn repeat_song(list: &mut Playlist, amount: &str) -> Result<()> {
	let amount = amount.parse::<usize>()?;
	let current_song = list.remaining.front().cloned().ok_or(NoFilePlaying)?;
	list.remaining = std::iter::repeat_n(current_song, amount)
		.chain(list.remaining.clone())
		.collect();
	Ok(())
}

pub fn save(ctx: &mut Context) -> Result<()> {
	if ctx.program_mode.can_save() {
		ctx.config.save(&ctx.savedata_path)?;
		ctx.files.save(&ctx.savedata_path)?;
		ctx.playlist
			.save(&ctx.config.current_playlist, &ctx.savedata_path)?;
		Ok(())
	} else {
		Err(SaveInWrongMode)
	}
}

/// Loads in the song at index 0 in the remaining song list as if it was just newly
/// loaded in, meaning for example the current progress is set back to 0.
/// This is different from just returning [`CommandReturn::ReloadFirstSong`],
/// which treats it as if the song had been played before it was returned.
pub fn load_in_first_song(ctx: &mut Context) -> CommandReturn {
	ctx.playlist.progress = Duration::ZERO;
	CommandReturn::ReloadFirstSong
}
