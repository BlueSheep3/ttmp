//! macros to make using commands less repetitive

use super::{
	error::{
		CommandError::{MacroAlreadyExists, MacroDoesNotExist},
		Result,
	},
	CommandReturn,
};
use crate::data::{config::Config, context::Context};

pub fn run_macro(ctx: &mut Context, name: &str, args: &[&str]) -> Result<CommandReturn> {
	let Some(commands) = ctx.config.macros.get(name) else {
		return Err(MacroDoesNotExist(name.to_owned()));
	};
	run_commands(ctx, commands.clone(), args)
}

pub fn run_macro_or(
	ctx: &mut Context,
	name: &str,
	args: &[&str],
	default: &str,
) -> Result<CommandReturn> {
	let commands = ctx.config.macros.get(name).map_or(default, |c| c);
	run_commands(ctx, commands.to_owned(), args)
}

pub fn run_commands(
	ctx: &mut Context,
	mut commands: String,
	args: &[&str],
) -> Result<CommandReturn> {
	// NOTE will do weird things when arguments contain $ symbols
	for (i, arg) in args.iter().enumerate().rev() {
		commands = commands.replace(&format!("${}", i), arg);
	}
	commands = commands.replace("$a", &args.join(" "));

	if commands.is_empty() {
		return Ok(CommandReturn::Nothing);
	}

	let commands = commands
		.split("; ")
		.map(|s| s.to_owned())
		.collect::<Vec<_>>();

	for cmd in commands {
		let state = super::match_input(&cmd, ctx)?;
		match state {
			CommandReturn::Nothing => (),
			CommandReturn::Quit => return Ok(state),
			CommandReturn::QuitNoSave => return Ok(state),
		}
	}
	Ok(CommandReturn::Nothing)
}

pub fn add_macro(config: &mut Config, name: &str, commands: &[&str]) -> Result<()> {
	if config.macros.contains_key(name) {
		return Err(MacroAlreadyExists(name.to_owned()));
	}
	let commands = commands.join(" ");
	config.macros.insert(name.to_owned(), commands);
	Ok(())
}

pub fn remove_macro(config: &mut Config, name: &str) -> Result<()> {
	if config.macros.remove(name).is_none() {
		return Err(MacroDoesNotExist(name.to_owned()));
	}
	Ok(())
}

pub fn change_macro(config: &mut Config, name: &str, commands: &[&str]) -> Result<()> {
	let m = config
		.macros
		.get_mut(name)
		.ok_or(MacroDoesNotExist(name.to_owned()))?;
	let commands = commands.join(" ");
	*m = commands;
	Ok(())
}

pub fn show_macros(config: &Config) {
	let mut macros = config.macros.iter().collect::<Vec<_>>();
	macros.sort_by_key(|&(name, _commands)| name);
	for (name, commands) in macros {
		println!("{} = {}", name, commands);
	}
}
