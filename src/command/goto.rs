use super::error::Result;
use crate::{
	config::Config,
	duration::{display_duration, display_duration_out_of, parse_duration},
	update_thread,
};
use rodio::Sink;

pub fn jump_to(config: &mut Config, sink: &Sink, duration: &[&str]) -> Result<()> {
	let duration = parse_duration(&duration.join(" "))?;
	config.progress = duration;
	config.dont_save_at = config.progress;
	update_thread::load_first_song(config, sink);
	Ok(())
}

pub fn jump_forward(config: &mut Config, sink: &Sink, duration: &[&str]) -> Result<()> {
	let duration = parse_duration(&duration.join(" "))?;
	config.progress += duration;
	config.dont_save_at = config.progress;
	update_thread::load_first_song(config, sink);
	Ok(())
}

pub fn display_progress(config: &Config) {
	if try_display_progress_out_of(config).is_none() {
		println!("{}", display_duration(config.progress));
	}
}

fn try_display_progress_out_of(config: &Config) -> Option<()> {
	let song_duration = config.get_current_duration()?;
	let current = config.progress;
	println!("{}", display_duration_out_of(current, song_duration));
	Some(())
}
