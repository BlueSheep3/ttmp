//! commands that act on the playing songs

use super::{
	error::{
		CommandError::{VolumeTooHigh, VolumeTooLow},
		Result,
	},
	misc, CommandReturn,
};
use crate::data::{config::StartPlayState, context::Context, playlist::Playlist};
use rand::seq::SliceRandom;

pub fn randomize(ctx: &mut Context) -> CommandReturn {
	ctx.playlist.remaining.shuffle(&mut rand::thread_rng());
	misc::load_in_first_song(ctx)
}

pub fn toggle_playing(ctx: &mut Context) {
	if ctx.sink.is_paused() {
		ctx.sink.play();
	} else {
		ctx.sink.pause();
	}
	// swapping the remembered play/pause state independantly of the sink's
	// play state, because the sink is always paused when no songs remain
	if let StartPlayState::Remember(p) = &mut ctx.config.start_play_state {
		*p ^= true;
	}
}

pub fn start_playing(ctx: &mut Context) {
	ctx.sink.play();
	if let StartPlayState::Remember(p) = &mut ctx.config.start_play_state {
		*p = true;
	}
}

pub fn pause_playing(ctx: &mut Context) {
	ctx.sink.pause();
	if let StartPlayState::Remember(p) = &mut ctx.config.start_play_state {
		*p = false;
	}
}

pub fn next_song(ctx: &mut Context) -> CommandReturn {
	if !ctx.playlist.remaining.is_empty() {
		ctx.playlist.remaining.remove(0);
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
	ctx.playlist.remaining.drain(..count);
	Ok(misc::load_in_first_song(ctx))
}

pub fn enforce_max(list: &mut Playlist, max: &str) -> Result<()> {
	let max = max.parse::<usize>()?;
	list.remaining.truncate(max);
	Ok(())
}

pub fn set_speed(ctx: &mut Context, speed: &str) -> Result<()> {
	let s = speed.parse::<f32>()?;
	ctx.config.speed = s;
	ctx.sink.set_speed(s);
	Ok(())
}

pub fn set_volume(ctx: &mut Context, volume: &str) -> Result<()> {
	let v = volume.parse::<f32>()?;
	if v < 0. {
		return Err(VolumeTooLow(v));
	}
	if v > 300. {
		return Err(VolumeTooHigh(v));
	}
	ctx.config.volume = v / 100.;
	ctx.sink.set_volume(ctx.config.volume);
	Ok(())
}

pub fn sort(ctx: &mut Context) -> CommandReturn {
	let Some(prev_current) = ctx.playlist.remaining.first().cloned() else {
		return CommandReturn::Nothing;
	};

	ctx.playlist.remaining.sort();

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}
