//! macros to make using commands less repetitive

use super::CommandReturn;
use crate::config::Config;
use rodio::Sink;

// returns true, if the program should quit
pub fn run_macro(config: &mut Config, sink: &Sink, name: &str, args: &[&str]) -> CommandReturn {
	let Some(commands) = config.macros.get(name) else {
		println!("Uknown Macro: {}", name);
		return CommandReturn::Nothing;
	};

	// NOTE will do weird things when arguments contain $ symbols
	let mut commands = commands.clone();
	for (i, arg) in args.iter().enumerate().rev() {
		commands = commands.replace(&format!("${}", i), arg);
	}
	commands = commands.replace("$a", &args.join(" "));

	if commands.is_empty() {
		return CommandReturn::Nothing;
	}

	let commands = commands
		.split("; ")
		.map(|s| s.to_owned())
		.collect::<Vec<_>>();

	for cmd in commands {
		let state = super::match_input(&cmd, sink, config);
		match state {
			CommandReturn::Nothing => (),
			CommandReturn::Quit => return state,
			CommandReturn::QuitNoSave => return state,
		}
	}
	CommandReturn::Nothing
}

pub fn add_macro(config: &mut Config, name: &str, commands: &[&str]) {
	if config.macros.contains_key(name) {
		println!("Macro already exists: {}", name);
	}
	let commands = commands.join(" ");
	config.macros.insert(name.to_owned(), commands);
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
