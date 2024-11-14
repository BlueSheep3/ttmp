mod error;
mod files;
mod filter;
mod goto;
mod help;
mod list;
mod macros;
mod misc;
mod play;
mod tag;

use self::error::{CommandError::UknownOrInvalidCommand, Result};
use crate::data::context::Context;

pub use self::macros::run_macro_or;

/// information to give the update thread after doing a Command
#[must_use]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum CommandReturn {
	#[default]
	Nothing,
	Quit,
	QuitNoSave,
	/// Reloads all data about the song at index 0 in the remaining song list.
	/// Note that this will also respect things like the current progress,
	/// so e.g. to skip a song you first need to set progress to 0 and then return this.
	ReloadFirstSong,
}

/// runs the command specified by `input` and may return information to the update thread
pub fn match_input(input: &str, ctx: &mut Context) -> Result<CommandReturn> {
	let input = input.split(' ').collect::<Vec<_>>();

	match input.as_slice() {
		["h" | "?" | "help"] => help::general(),
		["h" | "?" | "help", command] => help::specific(command)?,
		["q"] => return Ok(CommandReturn::Quit),
		["q!"] => return Ok(CommandReturn::QuitNoSave),
		["s"] => misc::save(ctx)?,
		["r"] => return Ok(misc::reset_remaining(ctx)),
		["echo", text @ ..] => misc::echo(text),
		["p"] => play::toggle_playing(ctx),
		["p+"] => play::start_playing(ctx),
		["p-"] => play::pause_playing(ctx),
		["pr"] => return Ok(play::randomize(ctx)),
		["pn"] => return Ok(play::next_song(ctx)),
		["pn", num] => return play::skip_songs(ctx, num),
		["pm", max] => play::enforce_max(&mut ctx.playlist, max)?,
		["ps", speed] => play::set_speed(ctx, speed)?,
		["pv", volume] => play::set_volume(ctx, volume)?,
		["pv"] => play::set_volume(ctx, "100")?,
		["po"] => return Ok(play::sort(ctx)),
		["pd", amount] => misc::repeat_song(&mut ctx.playlist, amount)?,
		["lg"] => list::get_current_name(ctx),
		["ln", name] => list::new_empty(ctx, name)?,
		["ld", name] => list::duplicate(ctx, name)?,
		["lc", name] => return list::copy_from(ctx, name),
		["lr", name] => list::remove(ctx, name)?,
		["ls", name] => return list::switch_to(ctx, name),
		["fte", tags @ ..] => return Ok(filter::tag_exists(ctx, tags)),
		["fta", tags @ ..] => return Ok(filter::tag_all(ctx, tags)),
		["ftn"] => return Ok(filter::no_tags(ctx)),
		["fsf", search @ ..] => return Ok(filter::search_full(ctx, search)),
		["fs", search @ ..] => return Ok(filter::search_file_name(ctx, search)),
		["fss", search @ ..] => return Ok(filter::filepath_starts_with(ctx, search)),
		["tlc"] => tag::show_current_tags(ctx)?,
		["tla"] => tag::show_all_tags(ctx),
		["tac", tag] => tag::add_tag_current(ctx, tag)?,
		["trc", tag] => tag::remove_tag_current(ctx, tag)?,
		["taa", tag] => tag::add_tag_remaining(ctx, tag)?,
		["tra", tag] => tag::remove_tag_remaining(ctx, tag)?,
		["g"] => return goto::jump_to(ctx, &["0"]),
		["g", duration @ ..] => return goto::jump_to(ctx, duration),
		["gf", duration @ ..] => return goto::jump_forward(ctx, duration),
		["gb", duration @ ..] => return goto::jump_backward(ctx, duration),
		["gd"] => goto::display_progress(ctx),
		["m", name, args @ ..] => return macros::run_macro(ctx, name, args),
		["ma", name, commands @ ..] => macros::add_macro(&mut ctx.config, name, commands)?,
		["mr", name] => macros::remove_macro(&mut ctx.config, name)?,
		["mc", name, commands @ ..] => macros::change_macro(&mut ctx.config, name, commands)?,
		["ml"] => macros::show_macros(&ctx.config),
		["dr"] => files::reload_files(ctx)?,
		["del"] => return files::delete_current(ctx),
		["dm", destination @ ..] => files::move_file(ctx, destination)?,
		["dp"] => files::show_full_path(ctx)?,
		["ds"] => files::show_directories(&ctx.config)?,
		[""] => return macros::run_macro_or(ctx, "@cmd_empty", &[], ""),
		[macro_name, args @ ..] if ctx.config.macros.contains_key(*macro_name) => {
			return macros::run_macro(ctx, macro_name, args);
		}
		_ => return Err(UknownOrInvalidCommand(input.join(" "))),
	}
	Ok(CommandReturn::Nothing)
}
