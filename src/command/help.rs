// Copyright (C) 2026, Arne Daude, Per Daude
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of 'ttmp': https://github.com/BlueSheep3/ttmp

//! handles all commands that show the user information
//! about commands and the program itself

use super::error::{CommandError::NoHelpAvailable, Result};

pub fn general(cmd_out: &mut String) {
	*cmd_out += "\
help CATEGORY  - show help for a category of commands

Categories of Subcommands:
first      - help for someone using this program for the first time
n, normal  - all commands in normal mode
p, play    - modify playing songs
l, list    - modify different playlists
f, filter  - filter the remaining songs
t, tag     - tags to filter songs
g, goto    - go to a time in the song
m, macro   - macros to easily do common things
e, event   - special macro names that get called automatically
d, dir     - commands concerning the file system

Misc Commands:
help, h, ? - show this help
q          - save and quit
q!         - quit without saving
q?         - save and quit even if errors occurred
s          - save
r          - reset the playlist (put all files in it)
redraw     - toggle whether screen redraws should happen
echo TEXT  - print out TEXT
";
}

pub fn specific(command: &str, cmd_out: &mut String) -> Result<()> {
	match command {
		"first" => first(cmd_out),
		"n" | "normal" => normal(cmd_out),
		"p" | "play" => play(cmd_out),
		"l" | "list" => list(cmd_out),
		"f" | "filter" => filter(cmd_out),
		"t" | "tag" => tags(cmd_out),
		"g" | "goto" => goto(cmd_out),
		"m" | "macro" => macros(cmd_out),
		"e" | "event" => event_macros(cmd_out),
		"d" | "dir" => file_system(cmd_out),
		_ => return Err(NoHelpAvailable(command.to_owned())),
	}
	Ok(())
}

fn first(cmd_out: &mut String) {
	*cmd_out += "\
The standard mode of this program is NORMAL MODE.
Here, pressing a key will instantly run that command.
You can also press ':' or 'c' to enter COMMAND MODE.
There, you can enter longer commands and then hit <enter> to execute that command.
Try typing ':help<enter>' to get a more general help page.
Some normal mode commands will also automatically enter command mode.

You may be wondering why you are not seeing any songs in your playlist.
This program does not automatically insert songs into its file list.
You must type in ':dr<enter>' to add all your songs to the file list
(it also removes any songs that were in the list, but are no longer in your music folder).
Then, press 'r' to add all songs in the file list to the current playlist.

The config directory is located at
 Linux :  $XDG_CONFIG_HOME/ttmp  or  $HOME/.config/ttmp
 MacOS :  /Users/USER/Library/Application Support/ttmp
Windows:  C:\\Users\\USER\\AppData\\Roaming\\ttmp

The data directory is located at
 Linux :  $XDG_DATA_HOME/ttmp  or  $HOME/.local/share/ttmp
 MacOS :  /Users/USER/Library/Application Support/ttmp
Windows:  C:\\Users\\USER\\AppData\\Roaming\\ttmp

You can select the directory that contains all your music
in the 'files.ron' file in the data directory.

The main way to organize songs here is to filter by tags.
You can add a new tag to the current song by pressing 't'.
You can then filter for all current songs with that tag by pressing 'f'.
So if you want to have all songs with the tag TAG
(including ones that are not in the current playlist),
you first press 'r' to get all songs and then
'f TAG<enter>' to filter out only the ones tagged TAG.
";
}

fn normal(cmd_out: &mut String) {
	*cmd_out += "\
This is the list of all the default keybinds in normal mode,
however you can rebind them in the config file.

:, ;, c      - enter command mode
?            - open a help page
q            - save and quit
S            - save
.            - repeat the last command manually entered in command mode

space        - pause/play
p            - pause
P            - play
right        - go forward 5 seconds
left         - go backwards 5 seconds
up           - increase volume by 5%
down         - decrease volume by 5%
0            - go to the start of the current song

r            - reset the playlist (put all files in it)
j            - go to the next song
k            - go to the previous song

f TAGS       - keeps all files that match any of TAGS
F TAGS       - keeps all files that match all of TAGS
s SEARCH     - keeps all files whose file name contains SEARCH
t TAG        - add TAG to the current file
T TAG        - remove TAG from the current file

l NAME       - switch to the list NAME
L            - get the names of all playlists

m NAME ARGS  - run macro with NAME and arguments ARGS
M            - lists all macros

These keybinds cannot be rebound and can be used in any mode:
ctrl c, esc  - go into normal mode and clear command output
ctrl r       - toggle whether screen redraws should happen
";
}

fn play(cmd_out: &mut String) {
	*cmd_out += "\
p         - toggle between play and pause
p+        - play
p-        - pause
pr        - randomize / shuffle playlist
pn NUM    - skips NUM songs
pp NUM    - goes back NUM songs
pcp       - clears all previous songs
px        - remove this song from the playlist
pm NUM    - set the maximum number of files to be played
ps SPEED  - set the playback speed to SPEED
pv VOLUME - set the playback volume to VOLUME
po        - order / sort the remaining songs
pd NUM    - repeat the currently playing song
";
}

fn list(cmd_out: &mut String) {
	*cmd_out += "\
lg        - get the names of all playlists
ln NAME   - create a new empty list with NAME
ld NAME   - duplicate all songs in the current playlist into the list NAME
lc NAME   - replace the current songs with a copy from NAME
la NAME   - append all songs from NAME to the end of the current playlist
lr NAME   - remove the list NAME
ls NAME   - switch to the list NAME
";
}

fn filter(cmd_out: &mut String) {
	*cmd_out += "\
fte TAGS   - keeps all files that match any of TAGS
fta TAGS   - keeps all files that match all of TAGS
ftn        - keeps all files that have no Tags
fsf SEARCH - keeps all files whose full path name contains SEARCH
fs  SEARCH - keeps all files whose file name contains SEARCH
fss SEARCH - keeps all files whose full path name starts with SEARCH

When filtering by tags, you can write '!FOO' to match songs that are not tagged 'FOO'.
";
}

fn tags(cmd_out: &mut String) {
	*cmd_out += "\
tlc     - display all tags of the current file
tla     - display all tags of all files
tac TAG - add TAG to the current file
trc TAG - remove TAG from the current file
taa TAG - add TAG to all remaining files
tra TAG - remove TAG from all remaining files
";
}

fn goto(cmd_out: &mut String) {
	*cmd_out += "\
g   TIME - go to TIME in the current song
gf  TIME - jumps forward by TIME
gb  TIME - jumps backward by TIME
gd       - display the progress of the current song

TIME can be specified using a space separated list of values with units.
For example '1m 20s' represents a duration of 1 minute and 20 seconds.
The allowed units are: h,m,s.
";
}

fn macros(cmd_out: &mut String) {
	*cmd_out += "\
A macro is formatted as follows: <command 1>; <command 2>; ...
and all instances of $0, $1, ... will be replaced with the arguments.
$a will insert all arguments separated by spaces.

If you want to write a ';' inside a macro you can use ';1',
which will be converted to ';' when the macro is called.
Similarly, ';2' converts to ';1' and ';3' converts to ';2'.

Event macros' names start with an @ symbol. Use 'help event' for more info.

m  NAME ARGS - run macro with NAME and arguments ARGS
ma NAME DEF  - add a macro with NAME that runs DEF
mr NAME      - remove a macro with NAME
mc NAME DEF  - change an existing macro with NAME to run DEF
ml           - lists all macros

When running 'm NAME ARGS' you can leave out the 'm '.
So just writing 'NAME ARGS' will also run the macro called 'NAME'.
";
}

fn event_macros(cmd_out: &mut String) {
	*cmd_out += "\
An event macro is a special kind of macro that starts with an @ symbol.
Event macros will automatically be called by the program, but can also be called manually.
Triggering an event from inside another event macro is not supported.
They must have on of the following names:

@cmd_empty   - entered an empty command
@song_start  - any song started
@song_end    - any song ended (called before @song_start)
@list_end    - the entire playlist ended (called after @song_end)
";
}

fn file_system(cmd_out: &mut String) {
	*cmd_out += "\
dr         - add new files to the config, and remove deleted ones
del        - trash the current song and remove it from the list of tracked files
dm  PATH   - move the current song to a new directory
dp         - show the full path of the current file
ds         - shows all directories
";
}
