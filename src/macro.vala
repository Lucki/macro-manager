namespace MacroManager {
	public class Macro : Object {
		internal static File? config_dir { get; default = File.new_build_filename(Environment.get_user_config_dir(), "macro-manager"); }
		internal static File? config_file { get; default = File.new_build_filename(config_dir.get_path(), "config.json"); }

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
					Process.spawn_async(config_dir.get_path(), { script.get_path() }, environment, SpawnFlags.SEARCH_PATH_FROM_ENVP, null, out pid);
					var stream = runtime_file.create(FileCreateFlags.PRIVATE);
					stream.write(((int) pid).to_string().data);
					stream.close();
				} else {
					Process.spawn_sync(config_dir.get_path(), { script.get_path() }, environment, SpawnFlags.SEARCH_PATH_FROM_ENVP, null);
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
			var xdo = Xdo.new();

			if (xdo == null || Xdo.get_focused_window_sane(xdo, out active_window) != Xdo.SUCCESS) {
				printerr("error: unable to get active window, using default macros\n");
				printerr("environment variables will be unavailable\n");
				_executable = "default";
			} else {
				var pid = Xdo.get_pid_window(xdo, active_window);

				if (pid == 0) {
					printerr("error: unable to get active window pid, using default macros\n");
				} else {
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

				if (xdo == null || Xdo.get_window_size(xdo, active_window, out active_window_width, out active_window_height) == Xdo.SUCCESS) {
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW_WIDTH", active_window_width.to_string(), true);
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_WINDOW_HEIGHT", active_window_height.to_string(), true);
				} else {
					printerr("error: unable to get active window size, environment variables will be unavailable\n");
				}

				// TODO: Location of active window?

				int? mouse_location_x = null;
				int? mouse_location_y = null;
				int? mouse_screen_num;
				if (xdo == null || Xdo.get_mouse_location(xdo, out mouse_location_x, out mouse_location_y, out mouse_screen_num) == Xdo.SUCCESS) {
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_MOUSE_X", mouse_location_x.to_string(), true);
					_environment = Environ.set_variable(environment, "MACRO_MANAGER_MOUSE_Y", mouse_location_y.to_string(), true);
				} else {
					printerr("error: unable to get mouse location, environment variables will be unavailable\n");
				}
			}

			Json.Node config_node;
			try {
				config_node = Json.from_string((string) config_file.load_bytes().get_data());
			} catch (Error e) {
				printerr("error: %s\n", e.message);

				return Return.ERROR;
			}

			if (config_node.get_node_type() == Json.NodeType.NULL || config_node.get_node_type() != Json.NodeType.OBJECT) {
				printerr("error: reading config file\n");

				return Return.ERROR;
			}

			while (script == null) {
				if (config_node.get_object().has_member(executable)) {
					var config_node_executable = config_node.get_object().get_member(executable);

					if (config_node_executable.get_node_type() == Json.NodeType.OBJECT
					    && config_node_executable.get_object().has_member(mset)) {
						var config_node_set = config_node_executable.get_object().get_member(mset);

						if (config_node_set.get_node_type() == Json.NodeType.OBJECT) {
							if (config_node_set.get_object().has_member(id)) {
								var config_node_id = config_node_set.get_object().get_member(id);

								if (config_node_id.get_node_type() == Json.NodeType.OBJECT
								    && config_node_id.get_object().has_member("script")) {
									_script = File.new_build_filename(config_dir.get_path(), config_node_id.get_object().get_string_member("script"));
									_toggle = config_node_id.get_object().get_boolean_member_with_default("toggle", false);
								}
							} else if (config_node_set.get_object().has_member("default_fallback")) {
								if (!config_node_set.get_object().get_boolean_member_with_default("default_fallback", false)) {
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

			if (script == null || !script.query_exists()) {
				printerr("error: invalid macro file for id \"%s\" in set \"%s\"\n", id, mset);

				return Return.ERROR;
			}

			return Return.SUCCESS;
		}
	}
}
