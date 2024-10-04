//! commands that act on the playing songs

use super::error::{
	CommandError::{NotEnoughSongsRemaining, VolumeTooHigh, VolumeTooLow},
	Result,
};
use crate::data::{context::Context, playlist::Playlist};
use rand::seq::SliceRandom;
use rodio::Sink;

pub fn randomize(ctx: &mut Context) {
	ctx.playlist.remaining.shuffle(&mut rand::thread_rng());
	next_song(ctx);
}

pub fn toggle_playing(sink: &Sink) {
	if sink.is_paused() {
		sink.play();
	} else {
		sink.pause();
	}
}

pub fn start_playing(sink: &Sink) {
	sink.play();
}

pub fn pause_playing(sink: &Sink) {
	sink.pause();
}

pub fn next_song(ctx: &mut Context) {
	ctx.playlist.dont_save_at = ctx.playlist.progress;
	ctx.sink.stop();
}

pub fn skip_songs(ctx: &mut Context, count: &str) -> Result<()> {
	let count = count.parse::<usize>()?;
	let max = ctx.playlist.remaining.len();
	if count > max {
		return Err(NotEnoughSongsRemaining);
	}
	if count == 0 {
		return Ok(());
	}
	ctx.playlist.remaining.drain(..count);
	// calling next_song() only actually skips over a song if it was just playing, and since
	// we just drained at least the first song, it will play the first song in the list
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
		next_song(ctx);
	}
}
