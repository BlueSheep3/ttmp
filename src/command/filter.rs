//! commands that filter the playlist based on some condition like tags or length

use super::{misc, CommandReturn};
use crate::data::context::Context;
use std::collections::HashSet;

fn tag_matches(file_tags: &HashSet<String>, match_tag: &str) -> bool {
	match_tag.strip_prefix('!').map_or_else(
		|| file_tags.contains(match_tag),
		|stripped| !file_tags.contains(stripped),
	)
}

pub fn tag_exists(ctx: &mut Context, tags: &[&str]) -> CommandReturn {
	let Some(prev_current) = ctx.playlist.remaining.first().cloned() else {
		return CommandReturn::Nothing;
	};

	ctx.playlist.remaining.retain(|file| {
		let file_tags = &ctx.files.entry(file.to_path_buf()).or_default().tags;
		tags.iter().any(|tag| tag_matches(file_tags, tag))
	});

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}

pub fn tag_all(ctx: &mut Context, tags: &[&str]) -> CommandReturn {
	let Some(prev_current) = ctx.playlist.remaining.first().cloned() else {
		return CommandReturn::Nothing;
	};

	ctx.playlist.remaining.retain(|file| {
		let file_tags = &ctx.files.entry(file.to_path_buf()).or_default().tags;
		tags.iter().all(|tag| tag_matches(file_tags, tag))
	});

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}

pub fn no_tags(ctx: &mut Context) -> CommandReturn {
	let Some(prev_current) = ctx.playlist.remaining.first().cloned() else {
		return CommandReturn::Nothing;
	};

	ctx.playlist.remaining.retain(|file| {
		let file_tags = &ctx.files.entry(file.to_path_buf()).or_default().tags;
		file_tags.is_empty()
	});

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}

pub fn search_full(ctx: &mut Context, search: &[&str]) -> CommandReturn {
	let Some(prev_current) = ctx.playlist.remaining.first().cloned() else {
		return CommandReturn::Nothing;
	};
	let search = search.join(" ").to_lowercase();

	ctx.playlist
		.remaining
		.retain(|file| file.to_string_lossy().to_lowercase().contains(&search));

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}

pub fn search_file_name(ctx: &mut Context, search: &[&str]) -> CommandReturn {
	let Some(prev_current) = ctx.playlist.remaining.first().cloned() else {
		return CommandReturn::Nothing;
	};
	let search = search.join(" ").to_lowercase();

	ctx.playlist.remaining.retain(|file| {
		file.file_name()
			.expect("couldn't get the filename")
			.to_string_lossy()
			.to_lowercase()
			.contains(&search)
	});

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}

pub fn filepath_starts_with(ctx: &mut Context, search: &[&str]) -> CommandReturn {
	let Some(prev_current) = ctx.playlist.remaining.first().cloned() else {
		return CommandReturn::Nothing;
	};
	let search = search.join(" ").to_lowercase();

	ctx.playlist
		.remaining
		.retain(|file| file.to_string_lossy().to_lowercase().starts_with(&search));

	if ctx.playlist.remaining.is_empty() || prev_current != ctx.playlist.remaining[0] {
		misc::load_in_first_song(ctx)
	} else {
		CommandReturn::Nothing
	}
}
