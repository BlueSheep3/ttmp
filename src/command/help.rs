//! handles all commands that show the user information
//! about commands and the program itself

pub fn general() {
	println!(
		"Misc Commands:
help, h, ? - Show this help
q          - Quit the Program
s          - Save the Config
r          - Reset the Playlist (put all files in it)
rf         - Add new files to the config, and remove deleted ones
max        - set the maximum number of files to be played
prog       - show the current progress
del        - delete the current song from your computer

Categories of Subcommands:
p          - modify playing songs
f          - filter the remaining songs
t          - tags to filter songs
m          - macros to easily do common things"
	);
}

pub fn specific(command: &str) {
	match command {
		"h" | "help" | "?" => general(),
		"q" | "q!" => quit(),
		"s" => save(),
		"r" => reset(),
		"rf" => reset_files(),
		"max" => max(),
		"prog" => progress(),
		"del" => delete(),
		"p" => play(),
		"f" => filter(),
		"t" => tags(),
		"m" => macros(),
		_ => println!("No Help available for: {}", command),
	}
}

fn quit() {
	println!(
		"q  - Quits the program and saves
q! - Quits the program without saving"
	);
}

fn save() {
	println!("s - Save the Config");
}

fn reset() {
	println!("r - Reset the Playlist (put all files in it)");
}

fn reset_files() {
	println!("rf - Add new files to the config, and remove deleted ones");
}

fn max() {
	println!("max NUM - set the maximum number of files to be played to NUM");
}

fn progress() {
	println!("prog - Show the current progress");
}

fn delete() {
	println!("del - delete the current song from your computer");
}

fn play() {
	println!(
		"p         - toggle between play and pause
p+        - play
p-        - pause
pr        - randomize / shuffle playlist
pn        - skip to the next song
ps SPEED  - set the playback speed to SPEED
pv VOLUME - set the playback volume to VOLUME
pl        - loop the remaining songs
pl-       - stop looping"
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
tar TAG - add TAG to all remaining Files
trr TAG - remove TAG from all remaining Files"
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
ml          - lists all Macros
<nothing>   - run the \"default\" Macro"
	);
}
