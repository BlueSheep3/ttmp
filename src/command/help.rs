//! handles all commands that show the user information
//! about commands and the program itself

use super::error::{CommandError::NoHelpAvailable, Result};

pub fn general() {
	println!(
		"h <category>  - show help for a category of commands

Misc Commands:
help, h, ? - Show this help
q          - Quit the Program
s          - Save the Config
r          - Reset the Playlist (put all files in it)
echo TEXT  - print out TEXT

Categories of Subcommands:
p, play    - modify playing songs
l, list    - modify differnt playlists
f, filter  - filter the remaining songs
t, tag     - tags to filter songs
g, goto    - goto a time in the song
m, macro   - macros to easily do common things
e, event   - special macro names that get called automatically
d, dir     - commands concerning the file system"
	);
}

pub fn specific(command: &str) -> Result<()> {
	match command {
		"p" | "play" => play(),
		"l" | "list" => list(),
		"f" | "filter" => filter(),
		"t" | "tag" => tags(),
		"g" | "goto" => goto(),
		"m" | "macro" => macros(),
		"e" | "event" => event_macros(),
		"d" | "dir" => file_system(),
		_ => return Err(NoHelpAvailable(command.to_owned())),
	}
	Ok(())
}

fn play() {
	println!(
		"p         - toggle between play and pause
p+        - play
p-        - pause
pr        - randomize / shuffle playlist
pn NUM    - skips NUM songs
pm NUM    - set the maximum number of files to be played
ps SPEED  - set the playback speed to SPEED
pv VOLUME - set the playback volume to VOLUME
po        - order / sort the remaining songs
pd NUM    - Repeat the currently playing song"
	);
}

fn list() {
	println!(
		"lg        - get the names of all playlists
ln NAME   - create a new empty list with NAME
ld NAME   - duplicate all songs in the current playlist into the list NAME
lc NAME   - replace the current songs with a copy from NAME
la NAME   - append all songs from NAME to the end of the current playlist
lr NAME   - remove the list NAME
ls NAME   - switch to the list NAME"
	);
}

fn filter() {
	println!(
		"fte TAGS   - Keeps all Files that match any of TAGS
fta TAGS   - Keeps all Files that match all of TAGS
ftn        - Keeps all Files that have no Tags
fsf SEARCH - Keeps all Files whose full path name contains SEARCH
fs  SEARCH - Keeps all Files whose file name contains SEARCH
fss SEARCH - Keeps all Files whose full path name starts with SEARCH"
	);
}

fn tags() {
	println!(
		"tlc     - Display all Tags of the current File
tla     - Display all Tags of all Files
tac TAG - add TAG to the current File
trc TAG - remove TAG from the current File
taa TAG - add TAG to all remaining Files
tra TAG - remove TAG from all remaining Files"
	);
}

fn goto() {
	println!(
		"g   TIME - go to TIME in the current Song
gf  TIME - jumps forward by TIME
gb  TIME - jumps backward by TIME
gd       - display the progress of the current Song"
	);
}

fn macros() {
	println!(
		"a Macro is formatted as follows: <command 1>; <command 2>; ...
and all instances of $0, $1, ... will be replaced with the arguments
$a will insert all arguments seperated by spaces

Event Macros' names start with an @ symbol. Use 'help event' for more info.

m NAME ARGS - run Macro with NAME and arguments ARGS
ma NAME DEF - add a Macro with NAME that runs DEF
mr NAME     - remove a Macro with NAME
mc NAME DEF - change an existing Macro with NAME to run DEF
ml          - lists all Macros"
	);
}

fn event_macros() {
	println!(
		"An Event Macro is a special kind of macro that starts with an @ symbol.
Event Macros will automatically be called by the program, but can also be called manually.
Triggering an Event from inside another Event Macro is not supported.
They must have on of the following names:

@cmd_empty   - entered an empty command
@song_start  - any song started
@song_end    - any song ended (called before @song_start)
@list_end    - the entire playlist ended (called after @song_end)"
	);
}

fn file_system() {
	println!(
		"dr         - Add new files to the config, and remove deleted ones
del        - delete the current song from your computer
dm  PATH   - move the current song to a new directory
dp         - show the full path of the file
ds         - shows all directories"
	);
}
