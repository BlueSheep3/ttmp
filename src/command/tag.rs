// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

//! commands that manipulate what tags the files have

use crate::{
	command::error::{
		CommandError::{NoFilePlaying, NotInFiles},
		Result,
	},
	data::context::Context,
};
use std::collections::HashSet;

pub fn show_current_tags(ctx: &mut Context) -> Result<()> {
	let current = ctx.playlist.remaining.front().ok_or(NoFilePlaying)?;
	let tags = &ctx
		.files
		.get(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	let tags = tags.iter().cloned().collect::<Vec<_>>().join(", ");
	ctx.cmd_out += &format!("current tags: {tags}\n");
	Ok(())
}

pub fn show_all_tags(ctx: &mut Context) {
	let tags = ctx
		.files
		.iter()
		.flat_map(|(_, data)| data.tags.iter())
		.cloned()
		.collect::<HashSet<_>>();
	let mut tags = tags.into_iter().collect::<Vec<_>>();
	tags.sort();
	ctx.cmd_out += &format!("all tags: {}\n", tags.join(", "));
}

pub fn add_tag_current(ctx: &mut Context, tag: &str) -> Result<()> {
	let current = ctx.playlist.remaining.front().ok_or(NoFilePlaying)?;
	let tags = &mut ctx
		.files
		.get_mut(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	tags.insert(tag.to_owned());
	Ok(())
}

pub fn remove_tag_current(ctx: &mut Context, tag: &str) -> Result<()> {
	let current = ctx.playlist.remaining.front().ok_or(NoFilePlaying)?;
	let tags = &mut ctx
		.files
		.get_mut(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	if !tags.remove(tag) {
		// TODO should this be an error?
		ctx.cmd_out += "The current File does not have that tag\n";
	}
	Ok(())
}

pub fn add_tag_remaining(ctx: &mut Context, tag: &str) -> Result<()> {
	for file in &mut ctx.playlist.remaining {
		let tags = &mut ctx
			.files
			.get_mut(file)
			.ok_or(NotInFiles(file.clone()))?
			.tags;
		tags.insert(tag.to_owned());
	}
	Ok(())
}

pub fn remove_tag_remaining(ctx: &mut Context, tag: &str) -> Result<()> {
	for file in &mut ctx.playlist.remaining {
		let tags = &mut ctx
			.files
			.get_mut(file)
			.ok_or(NotInFiles(file.clone()))?
			.tags;
		tags.remove(tag);
	}
	Ok(())
}
