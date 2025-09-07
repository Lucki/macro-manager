use clap::Parser;

/// Executes a script that is associated with the given SET and ID for the current active application.
///
/// When on X11 it exposes the following environment variables to called scripts:
/// MACRO_MANAGER_WINDOW
/// MACRO_MANAGER_WINDOW_BIN    // Should be the process name if known or "default" otherwise
/// MACRO_MANAGER_WINDOW_PID
/// MACRO_MANAGER_WINDOW_WIDTH
/// MACRO_MANAGER_WINDOW_HEIGHT
/// MACRO_MANAGER_MOUSE_X
/// MACRO_MANAGER_MOUSE_Y
/// MACRO_MANAGER_MOUSE_SCREEN
///
/// Config file location: "$XDG_CONFIG_HOME/macro-manager/config.toml"
/// Relative path script search location: "$XDG_DATA_HOME/macro-manager/"
///
/// Example config:
/// ~~~ toml
/// [default.set1.id3]
/// script = ["autoclicker.sh"]
/// toggle = true
///
/// ["program.exe".set1]
/// default_fallback = true
///
/// ["program.exe".set1.id8]
/// script = ["awesome_script1.sh"]
///
/// [firefox.m1]
/// g13.script = ["awesome_script2.sh", "arg1", "arg2"]
/// g14 = { script = ["subfolder/awesome_script3.sh"], toggle = true }
/// ~~~
///
/// Start/Stop (toggle) the "autoclicker.sh" script:
/// $ macro-manager set1 id3
///
/// Start "awesome_script2.sh" when in firefox:
/// $ macro-manager m1 g13
#[derive(Parser)]
#[clap(version, about, verbatim_doc_comment)]
struct Cli {
    /// Identifier for the current SET
    set: String,

    /// Identifier for the current ID
    id: String,
}

fn main() {
    let args = Cli::parse();
    let manager = macro_manager::Manager::new();
    let m = manager
        .get_macro(args.set, args.id)
        .expect("Failed initializing Macro");
    m.run();
}
