use super::play::{self, next_song};
use crate::config::Config;
use core::time::Duration;
use rodio::Sink;

pub fn reset_remaining(config: &mut Config, sink: &Sink) {
	config.remaining = config.files.keys().cloned().collect();
	config.looping_songs.clear();
	config.current_progress = Duration::ZERO;
	next_song(sink);
	if config.start_playing_immediately {
		play::start_playing(sink)
	}
}
