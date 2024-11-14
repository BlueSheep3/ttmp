//! commands that manipulate what tags the files have

use crate::{
	command::error::{
		CommandError::{NoFilePlaying, NotInFiles},
		Result,
	},
	data::context::Context,
};
use std::collections::HashSet;

pub fn show_current_tags(ctx: &Context) -> Result<()> {
	let current = ctx.playlist.remaining.first().ok_or(NoFilePlaying)?;
	let tags = &ctx
		.files
		.get(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	let tags = tags.iter().cloned().collect::<Vec<_>>().join(", ");
	println!("current tags: {}", tags);
	Ok(())
}

pub fn show_all_tags(ctx: &Context) {
	let tags = ctx
		.files
		.iter()
		.flat_map(|(_, data)| data.tags.iter())
		.cloned()
		.collect::<HashSet<_>>();
	let mut tags = tags.into_iter().collect::<Vec<_>>();
	tags.sort();
	println!("all tags: {}", tags.join(", "));
}

pub fn add_tag_current(ctx: &mut Context, tag: &str) -> Result<()> {
	let current = ctx.playlist.remaining.first().ok_or(NoFilePlaying)?;
	let tags = &mut ctx
		.files
		.get_mut(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	tags.insert(tag.to_owned());
	Ok(())
}

pub fn remove_tag_current(ctx: &mut Context, tag: &str) -> Result<()> {
	let current = ctx.playlist.remaining.first().ok_or(NoFilePlaying)?;
	let tags = &mut ctx
		.files
		.get_mut(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	if !tags.remove(tag) {
		println!("The current File does not have that tag");
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
