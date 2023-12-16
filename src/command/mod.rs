mod filter;
mod goto;
mod help;
mod macros;
mod misc;
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
		["h" | "?" | "help"] => help::general(),
		["h" | "?" | "help", command] => help::specific(command),
		["q"] => return CommandReturn::Quit,
		["q!"] => return CommandReturn::QuitNoSave,
		["s"] => config.save().expect("failed to save"),
		["r"] => misc::reset_remaining(config, sink),
		["rf"] => misc::reload_files(config),
		["del"] => misc::delete_current(config, sink),
		["max", max] => misc::enforce_max(config, max),
		["prog"] => misc::show_progress(config),
		["fm", destination @ ..] => misc::move_file(config, destination),
		["fp"] => misc::show_full_path(config),
		["p"] => play::toggle_playing(sink),
		["p+"] => play::start_playing(sink),
		["p-"] => play::pause_playing(sink),
		["pr"] => play::randomize(config, sink),
		["pn"] => play::next_song(sink),
		["ps", speed] => play::set_speed(config, sink, speed),
		["pv", volume] => play::set_volume(config, sink, volume),
		["pv"] => play::set_volume(config, sink, "100"),
		["pl"] => play::loop_remaining(config),
		["pl-"] => play::stop_looping(config),
		["fte", tags @ ..] => filter::tag_exists(config, sink, tags),
		["fta", tags @ ..] => filter::tag_all(config, sink, tags),
		["ftn"] => filter::no_tags(config, sink),
		["fsf", search @ ..] => filter::search_full(config, sink, search),
		["fs", search @ ..] => filter::search_file_name(config, sink, search),
		["fss", search] => filter::filepath_starts_with(config, sink, search),
		["tlc"] => tag::show_current_tags(config),
		["tla"] => tag::show_all_tags(config),
		["tac", tag] => tag::add_tag_current(config, tag),
		["trc", tag] => tag::remove_tag_current(config, tag),
		["taa", tag] => tag::add_tag_remaining(config, tag),
		["tra", tag] => tag::remove_tag_remaining(config, tag),
		["g"] => goto::jump_to(config, sink, "0"),
		["g", duration] => goto::jump_to(config, sink, duration),
		["gf", duration] => goto::jump_forward(config, sink, duration),
		["m", name, args @ ..] => return macros::run_macro(config, sink, name, args),
		["ma", name, commands @ ..] => macros::add_macro(config, name, commands),
		["mr", name] => macros::remove_macro(config, name),
		["ml"] => macros::show_macros(config),
		[""] => return macros::run_macro(config, sink, "default", &[]),
		[macro_name, args @ ..] if config.macros.contains_key(*macro_name) => {
			return macros::run_macro(config, sink, macro_name, args);
		}
		_ => invalid_command(&input),
	}
	CommandReturn::Nothing
}

fn invalid_command(input: &[&str]) {
	println!("Invalid Command: {}", input.join(" "));
}
