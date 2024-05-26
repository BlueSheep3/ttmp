//! commands that act on the playing songs

use super::error::{
	CommandError::{NotEnoughSongsRemaining, VolumeTooHigh, VolumeTooLow},
	Result,
};
use crate::config::Config;
use rand::seq::SliceRandom;
use rodio::Sink;

pub fn randomize(config: &mut Config, sink: &Sink) {
	config.remaining.shuffle(&mut rand::thread_rng());
	next_song(sink, config);
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

pub fn next_song(sink: &Sink, config: &mut Config) {
	config.dont_save_at = config.progress;
	sink.stop();
}

pub fn skip_songs(sink: &Sink, config: &mut Config, count: &str) -> Result<()> {
	let count = count.parse::<usize>()?;
	let max = config.remaining.len();
	if count > max {
		return Err(NotEnoughSongsRemaining);
	}
	if count == 0 {
		return Ok(());
	}
	config.remaining.drain(..count);
	// calling next_song() only actually skips over a song if it was just playing, and since
	// we just drained at least the first song, it will play the first song in the list
	next_song(sink, config);
	Ok(())
}

pub fn enforce_max(config: &mut Config, max: &str) -> Result<()> {
	let max = max.parse::<usize>()?;
	config.remaining.truncate(max);
	Ok(())
}

pub fn set_speed(config: &mut Config, sink: &Sink, speed: &str) -> Result<()> {
	let s = speed.parse::<f32>()?;
	config.speed = s;
	sink.set_speed(s);
	Ok(())
}

pub fn set_volume(config: &mut Config, sink: &Sink, volume: &str) -> Result<()> {
	let v = volume.parse::<f32>()?;
	if v < 0. {
		return Err(VolumeTooLow(v));
	}
	if v > 300. {
		return Err(VolumeTooHigh(v));
	}
	config.volume = v / 100.;
	sink.set_volume(config.volume);
	Ok(())
}

pub fn loop_remaining(config: &mut Config) {
	config.looping_songs = config.remaining.clone();
}

pub fn stop_looping(config: &mut Config) {
	config.looping_songs.clear();
}

pub fn sort(config: &mut Config, sink: &Sink) {
	let Some(prev_current) = config.remaining.first().cloned() else {
		return;
	};

	config.remaining.sort();

	if config.remaining.is_empty() || prev_current != config.remaining[0] {
		next_song(sink, config);
	}
}
