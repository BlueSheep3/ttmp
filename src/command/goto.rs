use crate::{config::Config, update_thread};
use rodio::Sink;
use std::time::Duration;

pub fn jump_to(config: &mut Config, sink: &Sink, duration: &str) {
	let duration = parse_duration(duration).unwrap();
	config.current_progress = duration;
	update_thread::load_first_song(config, sink);
}

pub fn jump_forward(config: &mut Config, sink: &Sink, duration: &str) {
	let duration = parse_duration(duration).unwrap();
	config.current_progress += duration;
	update_thread::load_first_song(config, sink);
}

fn parse_duration(duration_str: &str) -> Result<Duration, &'static str> {
	let mut duration = Duration::ZERO;
	for part in duration_str.split(' ') {
		let unit = part.chars().last().ok_or("empty duration part")?;
		if unit.is_ascii_digit() {
			// default to seconds
			let num = part.parse::<f32>().map_err(|_| "invalid duration part")?;
			duration += Duration::from_secs_f32(num);
			continue;
		}
		let num = part[0..part.len() - 1]
			.parse::<f32>()
			.map_err(|_| "invalid duration part")?;
		match unit {
			's' => duration += Duration::from_secs_f32(num),
			'm' => duration += Duration::from_secs_f32(num * 60.),
			'h' => duration += Duration::from_secs_f32(num * 60. * 60.),
			_ => return Err("unknown unit"),
		}
	}
	Ok(duration)
}
