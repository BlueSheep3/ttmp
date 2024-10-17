//! commands that act on the playing songs

use super::error::{
	CommandError::{VolumeTooHigh, VolumeTooLow},
	Result,
};
use crate::data::{config::StartPlayState, context::Context, playlist::Playlist};
use rand::seq::SliceRandom;

pub fn randomize(ctx: &mut Context) {
	ctx.playlist.remaining.shuffle(&mut rand::thread_rng());
	reload_current_song(ctx);
}

pub fn toggle_playing(ctx: &mut Context) {
	if ctx.sink.is_paused() {
		ctx.sink.play();
		if let StartPlayState::Remember(p) = &mut ctx.config.start_play_state {
			*p = true;
		}
	} else {
		ctx.sink.pause();
		if let StartPlayState::Remember(p) = &mut ctx.config.start_play_state {
			*p = false;
		}
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

pub fn next_song(ctx: &mut Context) {
	ctx.playlist.dont_save_at = ctx.playlist.progress;
	ctx.sink.stop();
}

pub fn reload_current_song(ctx: &mut Context) {
	// insert garbage data into first song, that will be instantly skipped by `next_song`.
	// spamming this function can make this song actually get recognized,
	// so by making it an actual song from the files, no garbage file data will be made.
	if let Some(first) = ctx.config.files.keys().next().cloned() {
		ctx.playlist.remaining.insert(0, first);
	}
	next_song(ctx);
}

pub fn skip_songs(ctx: &mut Context, count: &str) -> Result<()> {
	let count = count.parse::<usize>()?;
	let max = ctx.playlist.remaining.len();
	if count > max {
		ctx.playlist.remaining.clear();
		next_song(ctx);
		return Ok(());
	}
	if count == 0 {
		return Ok(());
	}
	// next_song will skip one song, so we remove 1 less from the playlist
	let count = count - 1;
	ctx.playlist.remaining.drain(..count);
	next_song(ctx);
	Ok(())
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

// pub fn loop_remaining(list: &mut Playlist) {
// 	list.looping_songs = list.remaining.clone();
// }

// pub fn stop_looping(list: &mut Playlist) {
// 	list.looping_songs.clear();
// }

pub fn sort(ctx: &mut Context) {
	let Some(prev_current) = ctx.playlist.remaining.first().cloned() else {
		return;
	};

	ctx.playlist.remaining.sort();

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		reload_current_song(ctx);
	}
}
