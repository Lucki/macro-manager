namespace MacroManager {
	enum Return {
		SUCCESS,
		ERROR,
		NO_CONFIG
	}

	private class Manager {
		// these are somehow null
		// internal static File? config_dir { get; default = File.new_build_filename(Environment.get_user_config_dir(), "macro-manager"); }
		// internal static File? config_file { get; default = File.new_build_filename(config_dir.get_path(), "config.json"); }

		private static int main(string[] args) {
			string macro_id = null;
			string macro_set = null;

			try {
				OptionEntry[] options = {
					// --id identifier
					{ "id", '\0', OptionFlags.NONE, OptionArg.STRING, ref macro_id, "Identifier for the current ID", "ID" },
					// --set identifier
					{ "set", '\0', OptionFlags.NONE, OptionArg.STRING, ref macro_set, "Identifier for the current SET", "SET" },

					// list terminator
					{ null }
				};

				var opt_context = new OptionContext();
				opt_context.set_help_enabled(true);
				opt_context.add_main_entries(options, null);
				opt_context.set_description("""Searches and executes a script that is associated with the given ID and SET for the active application.

It exposes the following environment variables to called scripts:
MACRO_MANAGER_WINDOW
MACRO_MANAGER_WINDOW_WIDTH
MACRO_MANAGER_WINDOW_HEIGHT
MACRO_MANAGER_MOUSE_X
MACRO_MANAGER_MOUSE_Y

Config file location: "$XDG_CONFIG_HOME/macro-manager/config.json"
Script search location: "$XDG_CONFIG_HOME/macro-manager/"

Example config:
~~~
{
	"default": {
		"set1": {
			"id3": {
				"script": "autoclicker.sh",
				"toggle": true
			}
		}
	},
	"program.exe": {
		"set1": {
			"default_fallback": true,
			"id8": {
				"script": "awesome_script1.sh"
			}
		}
	},
	"firefox" : {
		"m1" : {
			"g13" : {
				"script" : "awesome_script2.sh"
			}
		}
	}
}
~~~

Example autoclicker script:
~~~
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
""");
				opt_context.parse(ref args);
			} catch (OptionError e) {
				printerr("error: %s\n", e.message);
				printerr("Run '%s --help' to see a full list of available command line options.\n", args[0]);

				return Return.ERROR;
			}

			if (macro_id == null) {
				printerr("error: no identifier for macro\n");
				printerr("Run '%s --help' to see a full list of available command line options.\n", args[0]);

				return Return.ERROR;
			} else if (macro_set == null) {
				printerr("error: no identifier for set\n");
				printerr("Run '%s --help' to see a full list of available command line options.\n", args[0]);

				return Return.ERROR;
			}

			var macro = new Macro(macro_id, macro_set);

			return macro.run();
		}
	}
}
