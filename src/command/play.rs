//! commands that act on the playing songs

use crate::config::Config;
use rand::seq::SliceRandom;
use rodio::Sink;

pub fn randomize(config: &mut Config, sink: &Sink) {
	config.remaining.shuffle(&mut rand::thread_rng());
	next_song(sink);
}

pub fn toggle_playing(sink: &Sink) {
	if sink.is_paused() {
		sink.play();
	} else {
		sink.pause();
	}
}

pub fn start_playing(sink: &Sink) {
	sink.play();
}

pub fn pause_playing(sink: &Sink) {
	sink.pause();
}

pub fn next_song(sink: &Sink) {
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
		if v < 0. {
			println!("Volume can't be less than 0");
			return;
		}
		if v > 300. {
			println!("Volume can't be more than 300");
			return;
		}
		config.volume = v / 100.;
		sink.set_volume(config.volume);
	} else {
		println!("Invalid volume: {}", volume);
	}
}

pub fn loop_remaining(config: &mut Config) {
	if config.remaining.is_empty() {
		println!("No Songs to loop");
	} else {
		config.looping_songs = config.remaining.clone();
	}
}

pub fn stop_looping(config: &mut Config) {
	config.looping_songs.clear();
}
