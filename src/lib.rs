// use libxdo_sys::xdo_free;
use libxdo_sys::xdo_get_focused_window_sane;
use libxdo_sys::xdo_get_mouse_location;
use libxdo_sys::xdo_get_pid_window;
use libxdo_sys::xdo_get_window_size;
use libxdo_sys::xdo_new;
// use libxdo_sys::Struct_xdo;
use rustix::process::Signal;
use std::env;
use std::fs;
use std::io::Write;
use std::thread;
// use std::mem::replace;
use dbus::blocking::Connection;
use rustix::process::kill_process;
use serde_json;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::ptr::null;
use std::time::Duration;
use toml::Value;
use xdg::BaseDirectories;

pub struct Manager {
    xdg_dirs: BaseDirectories,
    config_table: toml::Table,
}

pub struct Macro {
    toggle: bool,
    script: PathBuf,
    xdg_dirs: BaseDirectories,
}

struct ConfigResult {
    script_file: PathBuf,
    toggle: bool,
}

impl Manager {
    pub fn new() -> Self {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("macro-manager")
            .expect("Unable to get user directories!");

        let config_table = fs::read_to_string(
            xdg_dirs
                .find_config_file("config.toml")
                .expect("Config file not found!"),
        )
        .expect("Failed reading config file!")
        .parse::<Value>()
        .expect("Couldn't parse config file!")
        .as_table()
        .expect("Config root is not a table!")
        .to_owned();

        return Self {
            xdg_dirs,
            config_table,
        };
    }

    pub fn get_macro(&self, set: String, id: String) -> Result<Macro, String> {
        return Macro::new(set, id, &self.config_table, &self.xdg_dirs);
    }

    // fn drop(&mut self) {
    //     if self.xdo.is_some() {

    //         let _xdo = replace(self.xdo.unwrap(), null() as Struct_xdo);
    //         unsafe {
    //             xdo_free(&mut _xdo);
    //         }
    //     }
    // }
}

impl Macro {
    fn new(
        set: String,
        id: String,
        config_table: &toml::Table,
        xdg_dirs: &BaseDirectories,
    ) -> std::result::Result<Self, String> {
        let mut executable = "default".to_owned();
        let tmp;

        // Check Wayland/X11
        // https://unix.stackexchange.com/a/559950
        match env::var("WAYLAND_DISPLAY") {
            Err(_) => {
                let xdo = unsafe { xdo_new(null()).as_mut() };
                if xdo.is_none() {
                    eprintln!("Failed initializing xdotool.");
                }

                if xdo.is_some() {
                    let xdo_safe = &**xdo.as_ref().unwrap();
                    let mut window_ret_init = 0;
                    let window_ret: *mut u64 = &mut window_ret_init;

                    match unsafe { xdo_get_focused_window_sane(xdo_safe, window_ret) } {
                        0 => {
                            match unsafe { window_ret.as_ref() } {
                                Some(val) => env::set_var("MACRO_MANAGER_WINDOW", val.to_string()),
                                None => (),
                            }

                            // X11 window pid
                            match unsafe { xdo_get_pid_window(xdo_safe, *window_ret) } {
                                0 => println!("Unable to get PID of active window!"),
                                window_pid @ _ => {
                                    env::set_var(
                                        "MACRO_MANAGER_WINDOW_PID",
                                        window_pid.to_string(),
                                    );

                                    let all_processes: Vec<procfs::process::Process> =
                                        procfs::process::all_processes()
                                            .expect("Can't read /proc")
                                            .filter_map(|p| match p {
                                                Ok(p) => Some(p),
                                                Err(e) => {
                                                    match e {
                                                        procfs::ProcError::NotFound(_) => None, // process vanished during iteration, ignore it
                                                        procfs::ProcError::Io(_e, _path) => None, // can match on path to decide if we can continue
                                                        x => {
                                                            println!("Can't read process due to error {x:?}"); // some unknown error
                                                            None
                                                        }
                                                    }
                                                }
                                            })
                                            .collect();

                                    for process in all_processes {
                                        if process.pid() == window_pid {
                                            tmp = match process.stat() {
                                                Ok(stat) => stat.comm,
                                                Err(e) => {
                                                    println!("Failed getting process name: {}", e);
                                                    "default".to_string()
                                                }
                                            };
                                            executable = tmp;
                                            break;
                                        }
                                    }
                                }
                            };

                            // X11 window size
                            let mut active_window_width_init = 0;
                            let mut active_window_height_init = 0;
                            let active_window_width: *mut u32 = &mut active_window_width_init;
                            let active_window_height: *mut u32 = &mut active_window_height_init;
                            match unsafe {
                                xdo_get_window_size(
                                    xdo_safe,
                                    *window_ret,
                                    active_window_width,
                                    active_window_height,
                                )
                            } {
                                0 => {
                                    match unsafe { active_window_width.as_ref() } {
                                        Some(val) => env::set_var(
                                            "MACRO_MANAGER_WINDOW_WIDTH",
                                            val.to_string(),
                                        ),
                                        None => (),
                                    }
                                    match unsafe { active_window_height.as_ref() } {
                                        Some(val) => env::set_var(
                                            "MACRO_MANAGER_WINDOW_HEIGHT",
                                            val.to_string(),
                                        ),
                                        None => (),
                                    }
                                }
                                _ => println!("Unable to get active window size"),
                            }

                            // X11 window location
                            // let mut window_init_x = 0;
                            // let mut window_init_y = 0;
                            // let mut window_init_screen = 0;
                            // let active_window_x: *mut i32 = &mut window_init_x;
                            // let active_window_y: *mut i32 = &mut window_init_x;
                            // let active_window_screen: *mut i32 = &mut window_init_x;
                            // xdo_get_win
                            // match xdo_get_window_location(xdo, *window_ret, active_window_x, active_window_y, active_window_screen) {
                            //     0 => {
                            //         env::set_var("MACRO_MANAGER_WINDOW_X", active_window_x.as_ref().unwrap().to_string());
                            //         env::set_var("MACRO_MANAGER_WINDOW_Y", active_window_y.as_ref().unwrap().to_string());
                            //         // env::set_var("MACRO_MANAGER_WINDOW_SCREEN", active_window_screen.as_ref().unwrap().as_ref().unwrap()..to_string());
                            //     },
                            //     _ => panic!("Unable to get window location"),
                            // }

                            println!("1");

                            // X11 mouse location
                            let mut mouse_init_x = 0;
                            let mut mouse_init_y = 0;
                            let mut mouse_init_screen = 0;
                            let mouse_location_x: *mut i32 = &mut mouse_init_x;
                            let mouse_location_y: *mut i32 = &mut mouse_init_y;
                            let mouse_location_screen: *mut i32 = &mut mouse_init_screen;
                            match unsafe {
                                xdo_get_mouse_location(
                                    xdo_safe,
                                    mouse_location_x,
                                    mouse_location_y,
                                    mouse_location_screen,
                                )
                            } {
                                0 => {
                                    match unsafe { mouse_location_x.as_ref() } {
                                        Some(val) => {
                                            env::set_var("MACRO_MANAGER_MOUSE_X", val.to_string())
                                        }
                                        None => (),
                                    }
                                    match unsafe { mouse_location_y.as_ref() } {
                                        Some(val) => {
                                            env::set_var("MACRO_MANAGER_MOUSE_Y", val.to_string())
                                        }
                                        None => (),
                                    }
                                    match unsafe { mouse_location_screen.as_ref() } {
                                        Some(val) => env::set_var(
                                            "MACRO_MANAGER_MOUSE_SCREEN",
                                            val.to_string(),
                                        ),
                                        None => (),
                                    }
                                }
                                _ => println!("Unable to get mouse location"),
                            }
                        }
                        _ => executable = "default".to_owned(),
                    }
                }
            }
            Ok(_) => {
                // TODO: Check wayland in general
                // FIXME: Gnome wayland hardcoded for now
                // Expect extension installed:
                // https://extensions.gnome.org/extension/4724/window-calls/
                // https://github.com/ickyicky/window-calls

                let connection = Connection::new_session().expect("Failed establishing connection");
                let proxy = connection.with_proxy(
                    "org.gnome.Shell",
                    "/org/gnome/Shell/Extensions/Windows",
                    Duration::from_millis(5000),
                );
                let (windows,): (String,) = proxy
                    .method_call("org.gnome.Shell.Extensions.Windows", "List", ())
                    .expect("No method result");
                let windows_json: serde_json::Value =
                    serde_json::from_str(&windows).expect("JSON was not well-formatted");

                let mut focused_window = None;
                for window in windows_json.as_array().unwrap() {
                    if window
                        .as_object()
                        .unwrap()
                        .get("focus")
                        .unwrap()
                        .as_bool()
                        .unwrap()
                        == true
                    {
                        focused_window = window.as_object();
                        break;
                    }
                }

                if focused_window.is_some() {
                    let window_id = focused_window.unwrap().get("id").unwrap().as_u64().unwrap();
                    env::set_var("MACRO_MANAGER_WINDOW", window_id.to_string());
                    env::set_var(
                        "MACRO_MANAGER_WINDOW_PID",
                        focused_window
                            .unwrap()
                            .get("pid")
                            .unwrap()
                            .as_i64()
                            .unwrap()
                            .to_string(),
                    );
                    executable = focused_window
                        .unwrap()
                        .get("wm_class")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned();

                    let (window_details,): (String,) = proxy
                        .method_call(
                            "org.gnome.Shell.Extensions.Windows",
                            "Details",
                            (u32::try_from(window_id).unwrap(),),
                        )
                        .expect("Failed method result");
                    let window_details_json: serde_json::Value =
                        serde_json::from_str(&window_details).expect("JSON was not well-formatted");

                    env::set_var(
                        "MACRO_MANAGER_WINDOW_WIDTH",
                        window_details_json
                            .as_object()
                            .unwrap()
                            .get("width")
                            .unwrap()
                            .as_i64()
                            .unwrap()
                            .to_string(),
                    );
                    env::set_var(
                        "MACRO_MANAGER_WINDOW_HEIGHT",
                        window_details_json
                            .as_object()
                            .unwrap()
                            .get("height")
                            .unwrap()
                            .as_i64()
                            .unwrap()
                            .to_string(),
                    );
                    env::set_var(
                        "MACRO_MANAGER_WINDOW_X",
                        window_details_json
                            .as_object()
                            .unwrap()
                            .get("x")
                            .unwrap()
                            .as_i64()
                            .unwrap()
                            .to_string(),
                    );
                    env::set_var(
                        "MACRO_MANAGER_WINDOW_Y",
                        window_details_json
                            .as_object()
                            .unwrap()
                            .get("y")
                            .unwrap()
                            .as_i64()
                            .unwrap()
                            .to_string(),
                    );

                    // Mouse info not available
                    // MACRO_MANAGER_MOUSE_X
                    // MACRO_MANAGER_MOUSE_Y
                    // MACRO_MANAGER_MOUSE_SCREEN
                }
            }
        }

        // Read toml
        let data_home = xdg_dirs.get_data_home();
        let config_result =
            match Self::read_config(config_table, &set, &id, &executable, &data_home) {
                Some(val) => val,
                None => {
                    if executable == "default" {
                        eprintln!("No macro found for SET \"{}\" and ID \"{}\".", set, id);
                        return Err("No macro found for SET and ID.".to_owned());
                    }

                    // Try again for default values
                    println!(
                        "No specific set found for \"{}\", trying \"default\"",
                        &executable
                    );
                    executable = "default".to_owned();

                    match Self::read_config(config_table, &set, &id, &executable, &data_home) {
                        Some(val) => val,
                        None => {
                            eprintln!("No macro found for SET \"{}\" and ID \"{}\".", set, id);
                            return Err("No macro found for SET and ID.".to_owned());
                        }
                    }
                }
            };

        if !config_result.script_file.try_exists().unwrap() {
            eprintln!(
                "Can't find script file \"{}\"",
                config_result.script_file.to_string_lossy()
            );
            return Err("Can't find script file.".to_owned());
        }

        // Should be the process name if known or "default" otherwise at this point
        env::set_var("MACRO_MANAGER_WINDOW_BIN", &executable);

        let xdg_macro = xdg::BaseDirectories::with_prefix(format!(
            "macro-manager/{}/{}/{}",
            executable, set, id
        ))
        .unwrap();

        Ok(Self {
            toggle: config_result.toggle,
            script: config_result.script_file,
            xdg_dirs: xdg_macro,
        })
    }

    pub fn run(&self) {
        let runtime_file_name = format!(
            "{}.pid",
            self.script
                .file_stem()
                .expect("Failed getting script file name")
                .to_str()
                .expect("Failed creating string form OsString")
        );
        let runtime_file = self.xdg_dirs.find_runtime_file(&runtime_file_name);

        if self.toggle {
            // Check runtime file
            match runtime_file {
                // Kill existing process and return if found
                Some(file) => {
                    let pid = fs::read_to_string(&file).expect("Failed reading PID file.");

                    // kill pid
                    match kill_process(
                        rustix::process::Pid::from_raw(pid.parse().unwrap())
                            .expect("Failed creating PID object"),
                        Signal::Term,
                    ) {
                        Err(err) => {
                            match err.raw_os_error() {
                                3 => (), // No such process
                                _ => panic!("Terminating existing process failed."),
                            };
                        }
                        Ok(()) => (),
                    }

                    // remove existing pid file
                    fs::remove_file(&file).expect("Removing PID file failed.");

                    return;
                }
                None => (),
            }
        }

        // Spawn process
        let mut process = Command::new(self.script.as_os_str())
            .spawn()
            .expect("Script failed to start.");

        // Return early if don't need to write the PID file
        if !self.toggle {
            thread::spawn(move || {
                process.wait().expect("Failed to wait on child process.");
            });
            return;
        }

        // Write PID to file
        let runtime_file = self
            .xdg_dirs
            .place_runtime_file(&runtime_file_name)
            .expect("Can't create runtime file. Started program can't be stopped again!");
        let mut pid_file = match File::create(&runtime_file) {
            Err(why) => panic!(
                "Couldn't create PID file \"{}\": {}",
                runtime_file.to_str().unwrap(),
                why
            ),
            Ok(file) => file,
        };

        match pid_file.write_all(process.id().to_string().as_bytes()) {
            Err(why) => panic!(
                "Couldn't write to PID file \"{}\": {}",
                runtime_file.to_str().unwrap(),
                why
            ),
            Ok(_) => {}
        }

        thread::spawn(move || {
            process.wait().expect("Failed to wait on child process.");
        });
    }

    fn read_config(
        config_table: &toml::Table,
        set: &str,
        id: &str,
        mut _executable: &str,
        data_home: &PathBuf,
    ) -> Option<ConfigResult> {
        if !config_table.contains_key(_executable) {
            return None;
        }

        let executable_table = config_table.get(_executable)?.as_table()?;
        if !executable_table.contains_key(set) {
            return None;
        }

        let set_table = executable_table.get(set)?.as_table()?;
        if !set_table.contains_key(id) && set_table.contains_key("default_fallback") {
            if !set_table.get("default_fallback")?.as_bool()? {
                // Prevent second run to respect user "false" by presetting this value to "default"
                _executable = "default";
            }

            return None;
        }

        let set_id_table = set_table.get(id)?.as_table()?;
        match set_id_table.get("script") {
            Some(value) => {
                let relative_script_path = value.as_str()?;

                // "toggle" is optional, fill missing value with "false"
                let toggle = match set_id_table.get("toggle") {
                    Some(val) => val.as_bool().unwrap_or_else(|| {
                        println!("Unable to read 'toggle' value, using 'false'");
                        false
                    }),
                    None => false,
                };

                return Some(ConfigResult {
                    // Using .join() here allows absolute paths to override the XDG_DATA_HOME location
                    script_file: Path::new(&data_home).join(relative_script_path),
                    toggle,
                });
            }
            None => (),
        }

        return None;
    }
}
