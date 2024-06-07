// use libxdo_sys::xdo_free;
use libxdo_sys::xdo_get_focused_window_sane;
use libxdo_sys::xdo_get_mouse_location;
use libxdo_sys::xdo_get_pid_window;
use libxdo_sys::xdo_get_window_size;
use libxdo_sys::xdo_new;
// use libxdo_sys::Struct_xdo;
use rustix::process::Signal;
use std::borrow::Borrow;
use std::fs;
use std::env;
use std::io::Write;
// use std::mem::replace;
use std::path::PathBuf;
use std::ptr::null;
use toml::Value;
use std::path::Path;
use std::process::Command;
use std::fs::File;
use rustix::process::kill_process;
use xdg::BaseDirectories;

pub struct Manager<'a> {
    xdg_dirs: BaseDirectories,
    config_table: toml::Table,
    xdo: Option<&'a mut libxdo_sys::Struct_xdo>,
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

impl Manager<'_> {
    pub fn new() -> Self {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("macro-manager")
            .expect("Unable to get user directories!");

        let config_table = fs::read_to_string(xdg_dirs
            .find_config_file("config.toml")
            .expect("Config file not found!"))
            .expect("Failed reading config file!")
            .parse::<Value>()
            .expect("Couldn't parse config file!")
            .as_table()
            .expect("Config root is not a table!")
            .to_owned();

        let xdo = unsafe { xdo_new(null()).as_mut() };
        if xdo.is_none() {
            eprintln!("Failed initializing xdotool.");
        }

        return Self {
            xdg_dirs,
            config_table,
            xdo,
        };
    }

    pub fn get_macro(&self, set: String, id: String) -> std::result::Result<Macro, &'static str> {
        return Macro::new(set, id, &self.config_table, self.xdg_dirs.borrow(), &self.xdo);
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
    fn new(set: String, id: String, config_table: &toml::Table, xdg_dirs: &BaseDirectories, xdo: &Option<&mut libxdo_sys::Struct_xdo>) -> std::result::Result<Self, &'static str> {
        let mut executable = "default";
        let tmp;

        // Check X11
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
                            env::set_var("MACRO_MANAGER_WINDOW_PID", window_pid.to_string());

                            let all_processes: Vec<procfs::process::Process> = procfs::process::all_processes()
                                .expect("Can't read /proc")
                                .filter_map(|p| match p {
                                    Ok(p) => Some(p),
                                    Err(e) => match e {
                                        procfs::ProcError::NotFound(_) => None, // process vanished during iteration, ignore it
                                        procfs::ProcError::Io(_e, _path) => None, // can match on path to decide if we can continue
                                        x => {
                                            println!("Can't read process due to error {x:?}"); // some unknown error
                                            None
                                        }
                                    },
                                })
                                .collect();

                            for process in all_processes {
                                if process.pid() == window_pid {
                                    tmp = match process.stat() {
                                        Ok(stat) => stat.comm,
                                        Err(e) => {
                                            println!("Failed getting process name: {}", e);
                                            "default".to_string()
                                        },
                                    };
                                    executable = tmp.as_ref();
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
                    match unsafe { xdo_get_window_size(xdo_safe, *window_ret , active_window_width, active_window_height) } {
                        0 => {
                            match unsafe { active_window_width.as_ref() } {
                                Some(val) => env::set_var("MACRO_MANAGER_WINDOW_WIDTH", val.to_string()),
                                None => (),
                            }
                            match unsafe { active_window_height.as_ref() } {
                                Some(val) => env::set_var("MACRO_MANAGER_WINDOW_HEIGHT", val.to_string()),
                                None => (),
                            }
                        },
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

                    // X11 mouse location
                    let mut mouse_init_x = 0;
                    let mut mouse_init_y = 0;
                    let mut mouse_init_screen = 0;
                    let mouse_location_x: *mut i32 = &mut mouse_init_x;
                    let mouse_location_y: *mut i32 = &mut mouse_init_y;
                    let mouse_location_screen: *mut i32 = &mut mouse_init_screen;
                    match unsafe { xdo_get_mouse_location(xdo_safe, mouse_location_x, mouse_location_y, mouse_location_screen) } {
                        0 => {
                            match unsafe { mouse_location_x.as_ref() } {
                                Some(val) => env::set_var("MACRO_MANAGER_MOUSE_X", val.to_string()),
                                None => (),
                            }
                            match unsafe { mouse_location_y.as_ref() } {
                                Some(val) => env::set_var("MACRO_MANAGER_MOUSE_Y", val.to_string()),
                                None => (),
                            }
                            match unsafe { mouse_location_screen.as_ref() } {
                                Some(val) => env::set_var("MACRO_MANAGER_MOUSE_SCREEN", val.to_string()),
                                None => (),
                            }
                        },
                        _ => println!("Unable to get mouse location"),
                    }
                },
                _ => executable = "default",
            }
        }

        // TODO: Check wayland

        // Read toml
        let data_home = xdg_dirs.get_data_home();
        let config_result = match Self::read_config(config_table, &set, &id, executable, &data_home) {
            Some(val) => val,
            None => {
                if executable == "default" {
                    eprintln!("No macro found for SET \"{}\" and ID \"{}\".", set, id);
                    return Err("No macro found for SET and ID.");
                }

                // Try again for default values
                executable = "default";
                match Self::read_config(config_table, &set, &id, executable, &data_home) {
                    Some(val) => val,
                    None => {
                        eprintln!("No macro found for SET \"{}\" and ID \"{}\".", set, id);
                        return Err("No macro found for SET and ID.");
                    }
                }
            }
        };

        if !config_result.script_file.try_exists().unwrap() {
            eprintln!("Can't find script file \"{}\"", config_result.script_file.to_string_lossy());
            return Err("Can't find script file.");
        }

        // Should be the process name if known or "default" otherwise at this point
        env::set_var("MACRO_MANAGER_WINDOW_BIN", executable);

        let xdg_macro = xdg::BaseDirectories::with_prefix(format!("macro-manager/{}/{}/{}", executable, set, id))
            .unwrap();

        Ok(
            Self {
                toggle: config_result.toggle,
                script: config_result.script_file,
                xdg_dirs: xdg_macro,
            }
        )
    }

    pub fn run(&self) {
        let runtime_file_name = format!("{}.pid",
                self.script
                    .file_stem()
                    .expect("Failed getting script file name")
                    .to_str()
                    .expect("Failed creating string form OsString"));
        let runtime_file = self.xdg_dirs.find_runtime_file(&runtime_file_name);

        if self.toggle {
            // Check runtime file
            match runtime_file {
                // Kill existing process and return if found
                Some(file) => {
                    let pid = fs::read_to_string(&file)
                        .expect("Failed reading PID file.");

                    // kill pid
                    match kill_process(rustix::process::Pid::from_raw(pid
                                .parse()
                                .unwrap())
                            .expect("Failed creating PID object"),
                        Signal::Term) {
                            Err(err) => {
                                match err.raw_os_error() {
                                    3 => (), // No such process
                                    _ => panic!("Terminating existing process failed."),
                                };
                            },
                            Ok(()) => (),
                        }

                    // remove existing pid file
                    fs::remove_file(&file)
                        .expect("Removing PID file failed.");

                    return;
                },
                None => (),
            }
        }

        // Spawn process
        let process = Command::new(self.script.as_os_str()).spawn()
            .expect("Script failed to start.");

        // Return early if don't need to write the PID file
        if !self.toggle {
            return;
        }

        // Write PID to file
        let runtime_file = self.xdg_dirs.place_runtime_file(&runtime_file_name)
            .expect("Can't create runtime file. Started program can't be stopped again!");
        let mut pid_file = match File::create(&runtime_file) {
            Err(why) => panic!("Couldn't create PID file \"{}\": {}", runtime_file.to_str().unwrap(), why),
            Ok(file) => file,
        };

        match pid_file.write_all(process.id().to_string().as_bytes()) {
            Err(why) => panic!("Couldn't write to PID file \"{}\": {}", runtime_file.to_str().unwrap(), why),
            Ok(_) => {},
        }
    }

    fn read_config(config_table: &toml::Table, set: &str, id: &str, mut _executable: &str, data_home: &PathBuf) -> Option<ConfigResult> {
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
                    Some(val) => val.as_bool()
                    .unwrap_or_else(|| {
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
            },
            None => (),
        }

        return None;
    }
}
