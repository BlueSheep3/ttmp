use super::{error::Result, CommandReturn};
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

pub fn display_progress(ctx: &Context) {
	if try_display_progress_out_of(ctx).is_none() {
		println!("{}", display_duration(ctx.playlist.progress));
	}
}

fn try_display_progress_out_of(ctx: &Context) -> Option<()> {
	let song_duration = ctx.get_current_duration()?;
	let current = ctx.playlist.progress;
	println!("{}", display_duration_out_of(current, song_duration));
	Some(())
}
