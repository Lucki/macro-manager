# Macro-Manager
Searches and executes a script that is associated with the given *ID* and *SET* for the active application.

If the current current executable isn't configured or a fallback is allowed for the current set it will try to use the default as fallback if available.

If the script config says the current script is toggelable it will stop already running instance of that script or otherwise start a new one.
This is intended for turning infinite running scripts on and off.

Make sure your script is marked as executable

* Config file location: `$XDG_CONFIG_HOME/macro-manager/config.toml`
* Script search location: `$XDG_DATA_HOME/macro-manager/`

It exposes the following environment variables from `xdotool` to called scripts:
~~~
MACRO_MANAGER_WINDOW
MACRO_MANAGER_WINDOW_PID
MACRO_MANAGER_WINDOW_WIDTH
MACRO_MANAGER_WINDOW_HEIGHT
MACRO_MANAGER_MOUSE_X
MACRO_MANAGER_MOUSE_Y
MACRO_MANAGER_MOUSE_SCREEN
~~~

## Installation
This manager requires `glib`, `gio`, `xdotool` and `tomlc99`.

Make requires `vala`, `meson` and `ninja`.
Make with `meson setup build` and then `meson compile -C build`.
Adjust options as needed.

Install with `meson install -C build`.
Adjust options as needed.

By default the library `libmacro-manager.so` will be installed which exposes the `Macro(string macro_id, string macro_set)` object with the `run()` method.
See [lib]macro-manager-[*].{h,gir,vapi} for details.

### Example config
~~~ toml
[default.set1.id3]
script = "autoclicker.sh"
toggle = true

["program.exe".set1]
default_fallback = true

["program.exe".set1.id8]
script = "awesome_script1.sh"

[firefox.m1]
g13.script = "awesome_script2.sh"
g14.script = "subfolder/awesome_script3.sh"
~~~

### Example autoclicker script
~~~ sh
#!/bin/bash

# define signal handler
term_handler() {
	# make sure we're not stuck in a mousedown event
	xdotool mouseup 1

	exit 0
}

# register signal handler
trap term_handler SIGTERM

while true; do
	# we 're not using xdotool click 1 here because that adds
	# a 12ms delay between mousedown and mouseup
	xdotool mousedown 1

	# this adjusts the click speed
	sleep 0.025

	xdotool mouseup 1
done
~~~

### Example emergency script - close all currently running scripts
~~~ sh
#!/bin/sh

find "$XDG_RUNTIME_DIR/macro-manager" -name "*.pid" -exec pkill -TERM -F {} \; -exec rm {} \;
~~~
