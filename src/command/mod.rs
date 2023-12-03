mod filter;
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

// returns true, if the program should quit
pub fn match_input(input: &str, sink: &Sink, config: &mut Config) -> CommandReturn {
	let input = input.split(' ').collect::<Vec<_>>();

	match input.as_slice() {
		["h" | "?" | "help"] => help(),
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
		["prog"] => println!("Progress: {:.02}", config.current_progress.as_secs_f32()),
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
		_ => println!("Invalid Command: {}", input.join(" ")),
	}
	CommandReturn::Nothing
}

pub fn help() {
	println!("Commands:");
	println!("h | ? - Show this help");
	println!("q - Quit the program");
	println!("q! - Quit the program without saving");
	println!("s - Save the config");
	println!("r - Reset the Playlist (put all files in it)");
	println!("rf - Add new files to the config, and remove deleted ones");
	println!("fte TAGS - Keeps all Files that match any of TAGS");
	println!("fta TAGS - Keeps all Files that match all of TAGS");
	println!("ftn - Keeps all Files that have no Tags");
	println!("fsf SEARCH - Keeps all Files whose full path name contains SEARCH");
	println!("fs SEARCH - Keeps all Files whose file name contains SEARCH");
	println!("p - Play / Pause");
	println!("pr - Randomize / Shuffle Playlist");
	println!("pn - Skip to the next Song");
	println!("ps SPEED - Set the playback speed");
	println!("pv VOLUME - Set the playback volume");
	println!("max NUM - Set the maximum number of files to be played");
	println!("prog - Show the current progress");
	println!("tlc - Display all Tags of the current File");
	println!("tla - Display all Tags of all Files");
	println!("tac TAG - add TAG to the current File");
	println!("trc TAG - remove TAG from the current File");
	println!("tar TAG - add TAG to all remaining Files");
	println!("trr TAG - remove TAG from all remaining Files");
	println!("m NAME ARGS - run Macro with NAME and arguments ARGS");
	println!("ma NAME STR - add a Macro with NAME that runs STR");
	println!("mr NAME - remove a Macro with NAME");
	println!("ml - lists all Macros");
}

fn reload_files(config: &mut Config) {
	config.reload_files().unwrap_or_else(|e| {
		println!("failed to add new files: {}", e);
	});
}
