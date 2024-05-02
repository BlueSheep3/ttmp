use std::io::{self, Seek, SeekFrom};
use std::path::PathBuf;
use std::{fs::File, time::Duration};

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
	let filepath = config.parent_path.join(config.remaining.first().unwrap());
	if let Ok(full_duration) = estimate_mp3_duration(filepath) {
		let percentage = config.current_progress.as_secs_f64() / full_duration.as_secs_f64();
		if !(0.0..=1.0).contains(&percentage) {
			println!("{}", percentage);
			return;
		}
		let total_length = 20;
		let bars = (percentage as f32 * total_length as f32).round() as usize;
		let dashes = total_length - bars;
		println!("{}|{}", "-".repeat(bars), "-".repeat(dashes));
	}
}

fn estimate_mp3_duration(file_path: PathBuf) -> io::Result<Duration> {
	let mut file = File::open(file_path)?;
	let size = file.seek(SeekFrom::End(0))?;
	calculate_duration(size)
}

fn calculate_duration(file_size: u64) -> io::Result<Duration> {
	const AVERAGE_BITRATE: u64 = 256; // in kbps
	let duration_seconds = (file_size * 8) / (AVERAGE_BITRATE * 1000);
	Ok(Duration::from_secs(duration_seconds))
}
