//! commands that act on the playing songs

use crate::config::Config;
use rand::seq::SliceRandom;
use rodio::Sink;
use std::time::Duration;

pub fn randomize(config: &mut Config, sink: &Sink) {
	config.remaining.shuffle(&mut rand::thread_rng());
	sink.stop();
}

pub fn toggle_playing(sink: &Sink) {
	if sink.is_paused() {
		sink.play();
	} else {
		sink.pause();
	}
}

pub fn reset(config: &mut Config, sink: &Sink) {
	config.remaining = config.files.keys().cloned().collect();
	config.current_progress = Duration::ZERO;
	sink.stop();
}

pub fn next(sink: &Sink) {
	sink.stop();
}

pub fn set_speed(config: &mut Config, sink: &Sink, speed: &str) {
	if let Ok(s) = speed.parse::<f32>() {
		config.speed = s;
		sink.set_speed(s);
	} else {
		println!("Invalid speed: {}", speed);
	}
}

pub fn set_volume(config: &mut Config, sink: &Sink, volume: &str) {
	if let Ok(v) = volume.parse::<f32>() {
		config.volume = v;
		sink.set_volume(v);
	} else {
		println!("Invalid volume: {}", volume);
	}
}

pub fn enforce_max(config: &mut Config, max: &str) {
	if let Ok(m) = max.parse::<usize>() {
		config.remaining.truncate(m);
	} else {
		println!("Invalid max: {}", max);
	}
}
