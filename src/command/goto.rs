// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

use super::{CommandReturn, error::Result};
use crate::{
	data::context::Context,
	duration::{display_duration, display_duration_out_of, parse_duration},
};

pub fn jump_to(ctx: &mut Context, duration: &[&str]) -> Result<CommandReturn> {
	let duration = parse_duration(&duration.join(" "))?;
	ctx.playlist.progress = duration;
	Ok(CommandReturn::ReloadFirstSong)
}

pub fn jump_forward(ctx: &mut Context, duration: &[&str]) -> Result<CommandReturn> {
	let duration = parse_duration(&duration.join(" "))?;
	ctx.playlist.progress += duration;
	Ok(CommandReturn::ReloadFirstSong)
}

pub fn jump_backward(ctx: &mut Context, duration: &[&str]) -> Result<CommandReturn> {
	let duration = parse_duration(&duration.join(" "))?;
	ctx.playlist.progress = ctx.playlist.progress.saturating_sub(duration);
	Ok(CommandReturn::ReloadFirstSong)
}

pub fn display_progress(ctx: &mut Context) {
	if try_display_progress_out_of(ctx).is_none() {
		ctx.cmd_out += &display_duration(ctx.playlist.progress);
		ctx.cmd_out.push('\n');
	}
}

fn try_display_progress_out_of(ctx: &mut Context) -> Option<()> {
	let song_duration = ctx.get_current_duration()?;
	let current = ctx.playlist.progress;
	ctx.cmd_out += &display_duration_out_of(current, song_duration);
	ctx.cmd_out.push('\n');
	Some(())
}
