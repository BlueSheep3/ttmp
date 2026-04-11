# ttmp

A minimal TUI music player that organizes songs using tags.

With most other music players, I've always found it restricting that you only really get
to pick one playlist. If you want to filter out specific songs in that playlist, too bad.
If you want to listen to songs from multiple specific playlists, too bad.

That's why this music player allows you to assign arbitrarily many tags to all your
songs, and you can the filter them based on any boolean expression.
So if you have the tags `a`, `b`, and `c`, you could filter all songs that have been
tagged `a` or `b`, but not `c`.



# Platform support

ttmp has been tested on Windows and Linux.
While it has not been tested on MacOS, it should also work.

The non-Windows versions are currently missing the IPC features,
but this should be fixed soon.



# Compiling from source

On Windows, you should just be able to run `cargo build`.

On Linux, you need to install the following dependencies: `pkg-config`, `alsa-lib`, `dbus`.
You then need to add the pkgconfig paths of `alsa-lib` and `dbus` to the
`$PKG_CONFIG_PATH` environment variable.
After that, running `cargo build` should work.
