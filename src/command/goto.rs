use super::error::{CommandError::InvalidDuration, Result};
use crate::{config::Config, update_thread};
use rodio::Sink;
use std::time::Duration;

pub fn jump_to(config: &mut Config, sink: &Sink, duration: &str) -> Result<()> {
	let duration = parse_duration(duration)?;
	config.current_progress = duration;
	update_thread::load_first_song(config, sink);
	Ok(())
}

pub fn jump_forward(config: &mut Config, sink: &Sink, duration: &str) -> Result<()> {
	let duration = parse_duration(duration)?;
	config.current_progress += duration;
	update_thread::load_first_song(config, sink);
	Ok(())
}

fn parse_duration(duration_str: &str) -> Result<Duration> {
	fn f32_to_duration(f: f32) -> Result<Duration> {
		Duration::try_from_secs_f32(f).map_err(|_| InvalidDuration("impossible duration"))
	}

	let mut duration = Duration::ZERO;
	for part in duration_str.split(' ') {
		let unit = part
			.chars()
			.last()
			.ok_or(InvalidDuration("empty duration part"))?;
		if unit.is_ascii_digit() {
			// default to seconds
			let num = part.parse::<f32>()?;
			duration += f32_to_duration(num)?;
			continue;
		}
		let num = part[0..part.len() - 1].parse::<f32>()?;
		match unit {
			's' => duration += f32_to_duration(num)?,
			'm' => duration += f32_to_duration(num * 60.)?,
			'h' => duration += f32_to_duration(num * 60. * 60.)?,
			_ => return Err(InvalidDuration("unknown unit")),
		}
	}
	Ok(duration)
}
