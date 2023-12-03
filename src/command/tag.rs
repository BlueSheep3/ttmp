//! commands that manipulate what tags the files have

use std::collections::HashSet;

use crate::config::Config;

pub fn show_current_tags(config: &Config) {
	let Some(current) = config.remaining.first() else {
		println!("No File currently playing");
		return;
	};
	let tags = &config.files.get(current).unwrap().tags;
	println!(
		"current tags: {}",
		tags.iter().cloned().collect::<Vec<_>>().join(", ")
	);
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

pub fn add_tag_current(config: &mut Config, tag: &str) {
	let Some(current) = config.remaining.first() else {
		println!("No File currently playing");
		return;
	};
	let tags = &mut config.files.get_mut(current).unwrap().tags;
	tags.insert(tag.to_string());
}

pub fn remove_tag_current(config: &mut Config, tag: &str) {
	let Some(current) = config.remaining.first() else {
		println!("No File currently playing");
		return;
	};
	let tags = &mut config.files.get_mut(current).unwrap().tags;
	if !tags.remove(tag) {
		println!("The current File does not have that tag");
	}
}

pub fn add_tag_remaining(config: &mut Config, tag: &str) {
	for file in &mut config.remaining {
		let tags = &mut config.files.get_mut(file).unwrap().tags;
		tags.insert(tag.to_string());
	}
}

pub fn remove_tag_remaining(config: &mut Config, tag: &str) {
	for file in &mut config.remaining {
		let tags = &mut config.files.get_mut(file).unwrap().tags;
		tags.remove(tag);
	}
}
