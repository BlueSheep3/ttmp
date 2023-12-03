//! macros to make using commands less repetitive

use crate::config::Config;
use rodio::Sink;

// returns true, if the program should quit
pub fn run_macro(config: &mut Config, sink: &Sink, name: &str, args: &[&str]) -> bool {
	let Some(commands) = config.macros.get(&name.to_string()) else {
		println!("Uknown Macro: {}", name);
		return false;
	};

	let mut commands = commands.clone();
	for (i, arg) in args.iter().enumerate() {
		commands = commands.replace(&format!("${}", i), arg);
	}

	let commands = commands
		.split("; ")
		.map(|s| s.to_string())
		.collect::<Vec<_>>();

	for cmd in commands {
		if super::match_input(&cmd, sink, config) {
			return true;
		}
	}
	false
}

pub fn add_macro(config: &mut Config, name: &str, commands: &[&str]) {
	if config.macros.contains_key(&name.to_string()) {
		println!("Macro already exists: {}", name);
	}
	let commands = commands.join(" ");
	config.macros.insert(name.to_string(), commands);
}

pub fn remove_macro(config: &mut Config, name: &str) {
	if config.macros.remove(name).is_none() {
		println!("Macro does not exist: {}", name);
	}
}

pub fn show_macros(config: &Config) {
	for (name, commands) in &config.macros {
		println!("{} = {}", name, commands);
	}
}
