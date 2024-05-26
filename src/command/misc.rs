use super::{
	error::{CommandError::NoFilePlaying, Result},
	play::{self, next_song},
};
use crate::config::Config;
use core::time::Duration;
use rodio::Sink;
use std::iter;

pub fn reset_remaining(config: &mut Config, sink: &Sink) {
	config.remaining = config.files.keys().cloned().collect();
	config.looping_songs.clear();
	config.progress = Duration::ZERO;
	next_song(sink, config);
	if config.start_playing_immediately {
		play::start_playing(sink)
	}
}

pub fn repeat_song(config: &mut Config, amount: &str) -> Result<()> {
	let amount = amount.parse::<usize>()?;
	let current_song = config.remaining.first().cloned().ok_or(NoFilePlaying)?;
	config.remaining = iter::repeat(current_song)
		.take(amount)
		.chain(config.remaining.clone())
		.collect();
	Ok(())
}
