namespace MacroManager {
	public class Macro:Object {
		internal static File? config_dir { get; default = File.new_build_filename(Environment.get_user_config_dir(), "macro-manager"); }
		internal static File? config_file { get; default = File.new_build_filename(config_dir.get_path(), "config.toml"); }
		internal static File? data_dir { get; default = File.new_build_filename(Environment.get_user_data_dir(), "macro-manager"); }

		private bool toggle { get; default = false; }
		private string executable { get; default = "default"; }
		private string id { get; }
		private string mset { get; }
		private string[] environment { get; default = Environ.get(); }
		private File file { get; }
		private File script { get; }
		private string? script_name {
			owned get {
				if (script == null) return null;

				return script.get_basename();
			}
		}
		private File? runtime_file {
			owned get {
				if (mset == null || id == null || script == null) return null;

				return File.new_build_filename(Environment.get_user_runtime_dir(), "macro-manager", executable, mset, id, script_name + ".pid");
			}
		}

		public Macro(string macro_id, string macro_set) {
			_id = macro_id;
			_mset = macro_set;
		}

		public int run() {

			int ret;
			if ((ret = get_details()) == Return.ERROR) {
				return Return.ERROR;
			} else if (ret == Return.NO_CONFIG) {
				return Return.SUCCESS;
			}

			try {
				if (toggle) {
					if (runtime_file.query_exists()) {
						Posix.pid_t pid = int.parse((string) runtime_file.load_bytes().get_data());
						Posix.kill(pid, Posix.Signal.TERM);
						runtime_file.delete ();

						return Return.SUCCESS;
					}

					Pid? pid = null;
					DirUtils.create_with_parents(runtime_file.get_parent().get_path(), 0700);
					Process.spawn_async(data_dir.get_path(), { script.get_path() }, environment, SpawnFlags.SEARCH_PATH_FROM_ENVP, null, out pid);
					var stream = runtime_file.create(FileCreateFlags.PRIVATE);
					stream.write(((int) pid).to_string().data);
					stream.close();
				} else {
					Pid? pid;
					Process.spawn_async(data_dir.get_path(), { script.get_path() }, environment, SpawnFlags.SEARCH_PATH_FROM_ENVP, null, out pid);
				}
			} catch (Error e) {
				printerr("error: %s\n", e.message);
				printerr("error: spawning child\n");

				return Return.ERROR;
			}

			return Return.SUCCESS;
		}

		private int get_details() {
			if (!config_file.query_exists()) {
				printerr("error: config file not found at \"%s\"\n", config_file.get_path());

				return Return.ERROR;
			}

			X.Window? active_window = null;
			var xdo = Xdo.Xdo.new();

			if (xdo == null || xdo.get_focused_window_sane(out active_window) != Xdo.SUCCESS) {
				printerr("error: unable to get active window, using default macros\n");
				printerr("environment variables will be unavailable\n");
				_executable = "default";
			} else {
				var pid = xdo.get_pid_window(active_window);

				if (pid == 0) {
					printerr("error: unable to get active window pid, using default macros\n");
				} else {
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW_PID", pid.to_string(), true);

					try {
						var comm_file = File.new_build_filename(Path.DIR_SEPARATOR_S, "proc", pid.to_string(), "comm");

						if (comm_file.query_exists()) {
							var str = (string) comm_file.load_bytes().get_data();
							// head -n 1
							_executable = str.substring(0, str.index_of("\n"));
						}
					} catch (Error e) {
						printerr("error: %s\n", e.message);
						printerr("error: unable to get active window pid executable, using default macros\n");
					}
				}

				_environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW", "%ld".printf((long) active_window), true);

				uint? active_window_width = null;
				uint? active_window_height = null;
				if (xdo.get_window_size(active_window, out active_window_width, out active_window_height) == Xdo.SUCCESS) {
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW_WIDTH", active_window_width.to_string(), true);
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW_HEIGHT", active_window_height.to_string(), true);
				} else {
					printerr("error: unable to get active window size, environment variables will be unavailable\n");
				}

				//  FIXME: Segmentation fault
				//  int? active_window_x = 0;
				//  int? active_window_y = 0;
				//  if (xdo.get_window_location(active_window, ref active_window_x, ref active_window_y) == Xdo.SUCCESS) {
				//  	_environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW_X", active_window_x.to_string(), true);
				//  	_environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW_Y", active_window_y.to_string(), true);
				//  	//  _environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW_SCREEN", active_window_screen.screen_number_of_screen().to_string(), true);
				//  } else {
				//  	printerr("error: unable to get window location, environment variables will be unavailable\n");
				//  }

				int? mouse_location_x = null;
				int? mouse_location_y = null;
				int? mouse_location_screen = null;
				if (xdo.get_mouse_location(out mouse_location_x, out mouse_location_y, out mouse_location_screen) == Xdo.SUCCESS) {
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_MOUSE_X", mouse_location_x.to_string(), true);
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_MOUSE_Y", mouse_location_y.to_string(), true);
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_MOUSE_SCREEN", mouse_location_screen.to_string(), true);
				} else {
					printerr("error: unable to get mouse location, environment variables will be unavailable\n");
				}
			}

			Toml.Table? config_node = null;
			uint8[] err = null;
			config_node = Toml.Table.from_file(FileStream.open(config_file.get_path(), "r"), out err);

			if (config_node == null) {
				printerr("error: reading config file\n");

				return Return.ERROR;
			}

			while (script == null) {
				if (config_node.contains(executable)) {
					var config_node_executable = config_node.get_table(executable);

					if (config_node_executable != null && config_node_executable.contains(mset)) {
						var config_node_set = config_node_executable.get_table(mset);

						if (config_node_set != null) {
							if (config_node_set.contains(id)) {
								var config_node_id = config_node_set.get_table(id);

								if (config_node_id != null && config_node_id.contains("script")) {
									var script_node = config_node_id.get_string("script");

									if (script_node != null) {
										_script = File.new_build_filename(data_dir.get_path(), script_node);
										_toggle = config_node_id.get_bool("toggle") ?? false;
									}
								}
							} else if (config_node_set.contains("default_fallback")) {
								bool fallback = config_node_set.get_bool("default_fallback") ?? false;
								if (!fallback) {
									_executable = "default";
								}
							}
						}
					}
				}

				if (script == null && executable != "default") {
					_executable = "default";
				} else if (script == null) {
					printerr("no macro found for set \"%s\" and id \"%s\"\n", mset, id);

					return Return.NO_CONFIG;
				}
			}

			config_node.free();

			if (script == null || !script.query_exists()) {
				printerr("error: invalid macro file for id \"%s\" in set \"%s\"\n", id, mset);

				return Return.ERROR;
			}

			return Return.SUCCESS;
		}
	}
}
