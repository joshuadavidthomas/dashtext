// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Handle --version before Tauri starts
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) {
        println!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    dashtext_lib::run();
}
