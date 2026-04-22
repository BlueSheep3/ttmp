// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

//! commands that act on the playing songs

use super::{
	CommandReturn,
	error::{
		CommandError::{VolumeTooHigh, VolumeTooLow},
		Result,
	},
	misc,
};
use crate::data::{context::Context, playlist::Playlist};
use rand::seq::SliceRandom;

pub fn randomize(ctx: &mut Context) -> CommandReturn {
	ctx.playlist
		.remaining
		.make_contiguous()
		.shuffle(&mut rand::rng());
	misc::load_in_first_song(ctx)
}

pub fn toggle_playing(ctx: &mut Context) -> Result<()> {
	if ctx.player.is_paused() {
		ctx.player.play();
	} else {
		ctx.player.pause();
	}
	// swapping the remembered play/pause state independantly of the player's
	// play state, because the player is always paused when no songs remain
	ctx.state.is_playing ^= true;
	ctx.update_media_progress()?;
	Ok(())
}

pub fn start_playing(ctx: &mut Context) -> Result<()> {
	ctx.player.play();
	ctx.state.is_playing = true;
	ctx.update_media_progress()?;
	Ok(())
}

pub fn pause_playing(ctx: &mut Context) -> Result<()> {
	ctx.player.pause();
	ctx.state.is_playing = false;
	ctx.update_media_progress()?;
	Ok(())
}

pub fn next_song(ctx: &mut Context) -> CommandReturn {
	if !ctx.playlist.remaining.is_empty() {
		ctx.playlist.next_song();
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}

pub fn skip_songs(ctx: &mut Context, count: &str) -> Result<CommandReturn> {
	let count = count.parse::<usize>()?;
	let len = ctx.playlist.remaining.len();
	let count = count.min(len);
	if count == 0 {
		return Ok(CommandReturn::Nothing);
	}
	let removed = ctx.playlist.remaining.drain(..count);
	ctx.playlist.previous.extend(removed);
	Ok(misc::load_in_first_song(ctx))
}

pub fn previous_song(ctx: &mut Context) -> CommandReturn {
	if !ctx.playlist.previous.is_empty() {
		ctx.playlist.previous_song();
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}

pub fn go_back_songs(ctx: &mut Context, count: &str) -> Result<CommandReturn> {
	let count = count.parse::<usize>()?;
	let len = ctx.playlist.previous.len();
	let count = count.min(len);
	if count == 0 {
		return Ok(CommandReturn::Nothing);
	}
	let removed = ctx
		.playlist
		.previous
		.drain((ctx.playlist.previous.len() - count)..)
		.rev();
	ctx.playlist.remaining.reserve(removed.len());
	for r in removed {
		ctx.playlist.remaining.push_front(r);
	}
	Ok(misc::load_in_first_song(ctx))
}

pub fn clear_previous(ctx: &mut Context) {
	ctx.playlist.previous.clear();
}

pub fn remove_current(ctx: &mut Context) {
	ctx.playlist.remaining.pop_front();
}

pub fn enforce_max(list: &mut Playlist, max: &str) -> Result<()> {
	let max = max.parse::<usize>()?;
	list.remaining.truncate(max);
	Ok(())
}

pub fn set_speed(ctx: &mut Context, speed: &str) -> Result<()> {
	let s = speed.parse::<f32>()?;
	ctx.state.speed = s;
	ctx.player.set_speed(s);
	Ok(())
}

pub fn set_volume(ctx: &mut Context, volume: &str) -> Result<()> {
	let v = volume.parse::<f32>()? / 100.;
	set_volume_float(ctx, v)
}

pub fn add_volume(ctx: &mut Context, add: &str) -> Result<()> {
	let a = add.parse::<f32>()? / 100.;
	set_volume_float(ctx, ctx.state.volume + a)
}

pub fn sub_volume(ctx: &mut Context, sub: &str) -> Result<()> {
	let s = sub.parse::<f32>()? / 100.;
	set_volume_float(ctx, ctx.state.volume - s)
}

fn set_volume_float(ctx: &mut Context, volume: f32) -> Result<()> {
	if volume < 0. {
		return Err(VolumeTooLow(volume));
	}
	if volume > 3. {
		return Err(VolumeTooHigh(volume));
	}
	ctx.state.volume = volume;
	ctx.player.set_volume(ctx.state.volume);
	ctx.update_media_volume()?;
	Ok(())
}

pub fn sort(ctx: &mut Context) -> CommandReturn {
	let Some(prev_current) = ctx.playlist.remaining.front().cloned() else {
		return CommandReturn::Nothing;
	};

	ctx.playlist.remaining.make_contiguous().sort();

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}
