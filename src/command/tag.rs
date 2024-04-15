//! commands that manipulate what tags the files have

use crate::{
	command::error::{
		CommandError::{NoFilePlaying, NotInFiles},
		Result,
	},
	config::Config,
};
use std::collections::HashSet;

pub fn show_current_tags(config: &Config) -> Result<()> {
	let current = config.remaining.first().ok_or(NoFilePlaying)?;
	let tags = &config
		.files
		.get(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	let tags = tags.iter().cloned().collect::<Vec<_>>().join(", ");
	println!("current tags: {}", tags);
	Ok(())
}

pub fn show_all_tags(config: &Config) {
	let tags = config
		.files
		.iter()
		.flat_map(|(_, data)| data.tags.iter())
		.cloned()
		.collect::<HashSet<_>>();
	let mut tags = tags.into_iter().collect::<Vec<_>>();
	tags.sort();
	println!("all tags: {}", tags.join(", "));
}

pub fn add_tag_current(config: &mut Config, tag: &str) -> Result<()> {
	let current = config.remaining.first().ok_or(NoFilePlaying)?;
	let tags = &mut config
		.files
		.get_mut(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	tags.insert(tag.to_owned());
	Ok(())
}

pub fn remove_tag_current(config: &mut Config, tag: &str) -> Result<()> {
	let current = config.remaining.first().ok_or(NoFilePlaying)?;
	let tags = &mut config
		.files
		.get_mut(current)
		.ok_or(NotInFiles(current.clone()))?
		.tags;
	if !tags.remove(tag) {
		println!("The current File does not have that tag");
	}
	Ok(())
}

pub fn add_tag_remaining(config: &mut Config, tag: &str) -> Result<()> {
	for file in &mut config.remaining {
		let tags = &mut config
			.files
			.get_mut(file)
			.ok_or(NotInFiles(file.clone()))?
			.tags;
		tags.insert(tag.to_owned());
	}
	Ok(())
}

pub fn remove_tag_remaining(config: &mut Config, tag: &str) -> Result<()> {
	for file in &mut config.remaining {
		let tags = &mut config
			.files
			.get_mut(file)
			.ok_or(NotInFiles(file.clone()))?
			.tags;
		tags.remove(tag);
	}
	Ok(())
}
