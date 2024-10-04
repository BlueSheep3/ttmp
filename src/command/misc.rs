use super::{
	error::{CommandError::NoFilePlaying, Result},
	play::{self, next_song},
};
use crate::data::{context::Context, playlist::Playlist};
use core::time::Duration;
use std::iter;

pub fn reset_remaining(ctx: &mut Context) {
	ctx.playlist.remaining = ctx.config.files.keys().cloned().collect();
	// config.looping_songs.clear();
	ctx.playlist.progress = Duration::ZERO;
	next_song(ctx);
	if ctx.config.start_play_state.should_play() {
		play::start_playing(&ctx.sink)
	}
}

pub fn repeat_song(list: &mut Playlist, amount: &str) -> Result<()> {
	let amount = amount.parse::<usize>()?;
	let current_song = list.remaining.first().cloned().ok_or(NoFilePlaying)?;
	list.remaining = iter::repeat(current_song)
		.take(amount)
		.chain(list.remaining.clone())
		.collect();
	Ok(())
}
