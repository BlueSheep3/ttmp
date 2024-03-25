use super::error::Result;
use crate::{
	config::Config,
	duration::{display_duration, parse_duration},
	update_thread,
};
use rodio::Sink;

pub fn jump_to(config: &mut Config, sink: &Sink, duration: &[&str]) -> Result<()> {
	let duration = parse_duration(&duration.join(" "))?;
	config.current_progress = duration;
	update_thread::load_first_song(config, sink);
	Ok(())
}

pub fn jump_forward(config: &mut Config, sink: &Sink, duration: &[&str]) -> Result<()> {
	let duration = parse_duration(&duration.join(" "))?;
	config.current_progress += duration;
	update_thread::load_first_song(config, sink);
	Ok(())
}

pub fn display_progress(config: &Config) {
	println!("{}", display_duration(config.current_progress));
}
