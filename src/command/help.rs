//! handles all commands that show the user information
//! about commands and the program itself

use super::error::{CommandError::NoHelpAvailable, Result};

pub fn general(cmd_out: &mut String) {
	*cmd_out += "\
h CATEGORY - show help for a category of commands

Misc Commands:
help, h, ? - Show this help
q          - Save and Quit the Program
q!         - Quit the Program without saving
s          - Save the Config
r          - Reset the Playlist (put all files in it)
redraw     - Toggle whether screen redraws should happen
echo TEXT  - print out TEXT

Categories of Subcommands:
first      - help for someone using this program for the first time
n, normal  - all commands in normal mode
p, play    - modify playing songs
l, list    - modify differnt playlists
f, filter  - filter the remaining songs
t, tag     - tags to filter songs
g, goto    - goto a time in the song
m, macro   - macros to easily do common things
e, event   - special macro names that get called automatically
d, dir     - commands concerning the file system
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
You must type in ':dr<enter>' add all your songs to the file list
(it also removes any songs that were in the list, but are no longer in your music folder).
Then, press 'r' to add all songs in the file list to the current playlist.

You can change the path to your music folder in the config, which is located at
 Linux :  ~/.local/share/musicplayer/config.ron
 MacOS :  /Users/USER/Library/Application Support/musicplayer/config.ron
Windows:  C:\\Users\\USER\\AppData\\Roaming\\musicplayer\\config.ron

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
:, ;, c      - enter command mode
?            - open this help page
q            - save and quit
S            - save

space        - pause/play
p            - pause
P            - play
right        - go forward 5 seconds
left         - go backwards 5 seconds
up           - increase volume by 5%
down         - decrease volume by 5%
0            - go to the start of the current song

r            - Reset the Playlist (put all files in it)
j            - go to the next song

f TAGS       - Keeps all Files that match any of TAGS
F TAGS       - Keeps all Files that match all of TAGS
s SEARCH     - Keeps all Files whose file name contains SEARCH
t TAG        - add TAG to the current File
T TAG        - remove TAG from the current File

l NAME       - switch to the list NAME
L            - get the names of all playlists

m            - lists all Macros
M NAME DEF   - add a Macro with NAME that runs DEF
";
}

fn play(cmd_out: &mut String) {
	*cmd_out += "\
p         - toggle between play and pause
p+        - play
p-        - pause
pr        - randomize / shuffle playlist
pn NUM    - skips NUM songs
pm NUM    - set the maximum number of files to be played
ps SPEED  - set the playback speed to SPEED
pv VOLUME - set the playback volume to VOLUME
po        - order / sort the remaining songs
pd NUM    - Repeat the currently playing song
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
fte TAGS   - Keeps all Files that match any of TAGS
fta TAGS   - Keeps all Files that match all of TAGS
ftn        - Keeps all Files that have no Tags
fsf SEARCH - Keeps all Files whose full path name contains SEARCH
fs  SEARCH - Keeps all Files whose file name contains SEARCH
fss SEARCH - Keeps all Files whose full path name starts with SEARCH
";
}

fn tags(cmd_out: &mut String) {
	*cmd_out += "\
tlc     - Display all Tags of the current File
tla     - Display all Tags of all Files
tac TAG - add TAG to the current File
trc TAG - remove TAG from the current File
taa TAG - add TAG to all remaining Files
tra TAG - remove TAG from all remaining Files
";
}

fn goto(cmd_out: &mut String) {
	*cmd_out += "\
g   TIME - go to TIME in the current Song
gf  TIME - jumps forward by TIME
gb  TIME - jumps backward by TIME
gd       - display the progress of the current Song
";
}

fn macros(cmd_out: &mut String) {
	*cmd_out += "\
a Macro is formatted as follows: <command 1>; <command 2>; ...
and all instances of $0, $1, ... will be replaced with the arguments
$a will insert all arguments seperated by spaces

Event Macros' names start with an @ symbol. Use 'help event' for more info.

m NAME ARGS - run Macro with NAME and arguments ARGS
ma NAME DEF - add a Macro with NAME that runs DEF
mr NAME     - remove a Macro with NAME
mc NAME DEF - change an existing Macro with NAME to run DEF
ml          - lists all Macros
";
}

fn event_macros(cmd_out: &mut String) {
	*cmd_out += "\
An Event Macro is a special kind of macro that starts with an @ symbol.
Event Macros will automatically be called by the program, but can also be called manually.
Triggering an Event from inside another Event Macro is not supported.
They must have on of the following names:

@cmd_empty   - entered an empty command
@song_start  - any song started
@song_end    - any song ended (called before @song_start)
@list_end    - the entire playlist ended (called after @song_end)
";
}

fn file_system(cmd_out: &mut String) {
	*cmd_out += "\
dr         - Add new files to the config, and remove deleted ones
del        - delete the current song from your computer
dm  PATH   - move the current song to a new directory
dp         - show the full path of the file
ds         - shows all directories
";
}
