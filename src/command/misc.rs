use super::{
	error::{
		CommandError::{NoFilePlaying, SaveInWrongMode},
		Result,
	},
	play,
};
use crate::data::{context::Context, playlist::Playlist};
use std::iter;

pub fn reset_remaining(ctx: &mut Context) {
	ctx.playlist.remaining = ctx.config.files.keys().cloned().collect();
	play::reload_current_song(ctx);
	if ctx.config.start_play_state.should_play() {
		play::start_playing(ctx)
	}
}

pub fn echo(text: &[&str]) {
	println!("{}", text.join(" "));
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

pub fn save(ctx: &mut Context) -> Result<()> {
	if ctx.program_mode.can_save() {
		ctx.config.save()?;
		ctx.playlist.save(&ctx.config.current_playlist)?;
		Ok(())
	} else {
		Err(SaveInWrongMode)
	}
}
