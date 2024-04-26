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

Categories of Subcommands:
p          - modify playing songs
f          - filter the remaining songs
t          - tags to filter songs
g          - goto a time in the song
m          - macros to easily do common things
d          - commands concerning the file system"
	);
}

pub fn specific(command: &str) -> Result<()> {
	match command {
		"p" => play(),
		"f" => filter(),
		"t" => tags(),
		"g" => goto(),
		"m" => macros(),
		"d" => file_system(),
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
pl        - loop the remaining songs
pl-       - stop looping
po        - order / sort the remaining songs"
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
gd       - display the progress of the current Song"
	);
}

fn macros() {
	println!(
		"a Macro is formatted as follows: <command 1>; <command 2>; ...
and all instances of $0, $1, ... will be replaced with the arguments
$a will insert all arguments seperated by spaces

m NAME ARGS - run Macro with NAME and arguments ARGS
ma NAME STR - add a Macro with NAME that runs STR
mr NAME     - remove a Macro with NAME
mc NAME STR - change an existing Macro with NAME to run STR
ml          - lists all Macros
<nothing>   - run the \"default\" Macro"
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
