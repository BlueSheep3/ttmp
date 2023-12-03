mod filter;
mod help;
mod macros;
mod play;
mod tag;

use crate::config::Config;
use rodio::Sink;

/// information to give the update thread after doing a Command
#[derive(Debug, Default, Clone, PartialEq)]
pub enum CommandReturn {
	#[default]
	Nothing,
	Quit,
	QuitNoSave,
}

/// runs the command specified by `input` and may return information to the update thread
pub fn match_input(input: &str, sink: &Sink, config: &mut Config) -> CommandReturn {
	let input = input.split(' ').collect::<Vec<_>>();

	match input.as_slice() {
		["h" | "?" | "help"] => help::help(),
		["q"] => return CommandReturn::Quit,
		["q!"] => return CommandReturn::QuitNoSave,
		["s"] => config.save().unwrap(),
		["r"] => play::reset(config, sink),
		["rf"] => reload_files(config),
		["fte", tags @ ..] => filter::tag_exists(config, sink, tags),
		["fta", tags @ ..] => filter::tag_all(config, sink, tags),
		["ftn"] => filter::no_tags(config, sink),
		["fsf", search @ ..] => filter::search_full(config, sink, search),
		["fs", search @ ..] => filter::search_file_name(config, sink, search),
		["p"] => play::toggle_playing(sink),
		["pr"] => play::randomize(config, sink),
		["pn"] => play::next(sink),
		["ps", speed] => play::set_speed(config, sink, speed),
		["pv", volume] => play::set_volume(config, sink, volume),
		["max", max] => play::enforce_max(config, max),
		["prog"] => show_progress(config),
		["tlc"] => tag::show_current_tags(config),
		["tla"] => tag::show_all_tags(config),
		["tac", tag] => tag::add_tag_current(config, tag),
		["trc", tag] => tag::remove_tag_current(config, tag),
		["taa", tag] => tag::add_tag_remaining(config, tag),
		["tra", tag] => tag::remove_tag_remaining(config, tag),
		["m", name, args @ ..] => return macros::run_macro(config, sink, name, args),
		["ma", name, commands @ ..] => macros::add_macro(config, name, commands),
		["mr", name] => macros::remove_macro(config, name),
		["ml"] => macros::show_macros(config),
		_ => invalid_command(&input),
	}
	CommandReturn::Nothing
}

fn reload_files(config: &mut Config) {
	config.reload_files().unwrap_or_else(|e| {
		println!("failed to add new files: {}", e);
	});
}

fn show_progress(config: &Config) {
	println!("Progress: {:.02}", config.current_progress.as_secs_f32());
}

fn invalid_command(input: &[&str]) {
	println!("Invalid Command: {}", input.join(" "));
}
