//! commands that filter the playlist based on some condition like tags or length

use crate::config::Config;
use rodio::Sink;
use std::collections::HashSet;

fn tag_matches(file_tags: &HashSet<String>, match_tag: &str) -> bool {
	match_tag.strip_prefix('!').map_or_else(
		|| file_tags.contains(&match_tag.to_string()),
		|stripped| !file_tags.contains(&stripped.to_string()),
	)
}

pub fn tag_exists(config: &mut Config, sink: &Sink, tags: &[&str]) {
	if config.remaining.is_empty() {
		return;
	}
	let prev_current = config.remaining[0].clone();

	config.remaining.retain(|file| {
		let file_tags = &config.files.entry(file.to_path_buf()).or_default().tags;
		tags.iter().any(|tag| tag_matches(file_tags, tag))
	});

	if config.remaining.is_empty() || prev_current != config.remaining[0] {
		sink.stop();
	}
}

pub fn tag_all(config: &mut Config, sink: &Sink, tags: &[&str]) {
	if config.remaining.is_empty() {
		return;
	}
	let prev_current = config.remaining[0].clone();

	config.remaining.retain(|file| {
		let file_tags = &config.files.entry(file.to_path_buf()).or_default().tags;
		tags.iter().all(|tag| tag_matches(file_tags, tag))
	});

	if config.remaining.is_empty() || prev_current != config.remaining[0] {
		sink.stop();
	}
}

pub fn no_tags(config: &mut Config, sink: &Sink) {
	if config.remaining.is_empty() {
		return;
	}
	let prev_current = config.remaining[0].clone();

	config.remaining.retain(|file| {
		let file_tags = &config.files.entry(file.to_path_buf()).or_default().tags;
		file_tags.is_empty()
	});

	if config.remaining.is_empty() || prev_current != config.remaining[0] {
		sink.stop();
	}
}

pub fn search_full(config: &mut Config, sink: &Sink, search: &[&str]) {
	if config.remaining.is_empty() {
		return;
	}
	let prev_current = config.remaining[0].clone();

	let search = search.join(" ").to_lowercase();

	config
		.remaining
		.retain(|file| file.to_string_lossy().to_lowercase().contains(&search));

	if config.remaining.is_empty() || prev_current != config.remaining[0] {
		sink.stop();
	}
}

pub fn search_file_name(config: &mut Config, sink: &Sink, search: &[&str]) {
	if config.remaining.is_empty() {
		return;
	}
	let prev_current = config.remaining[0].clone();

	let search = search.join(" ").to_lowercase();

	config.remaining.retain(|file| {
		file.file_name()
			.unwrap()
			.to_string_lossy()
			.to_lowercase()
			.contains(&search)
	});

	if config.remaining.is_empty() || prev_current != config.remaining[0] {
		sink.stop();
	}
}
