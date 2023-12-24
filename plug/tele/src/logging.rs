use std::{
    fs::File,
    sync::Mutex,
};

use tracing::{Level, info};

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn tk_init_logging(logPath: String) -> bool;
        fn tk_init_logging_stdout() -> bool;
        fn tk_log_info(message: String);
    }
}

pub fn tk_log_info(message: String) {
    info!(message);
}

pub fn tk_init_logging_stdout() -> bool {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_ansi(false)
        .with_thread_ids(true)
        .finish();

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("Setting global tracing subscriber failed.");
        return false;
    }
    true
}

pub fn tk_init_logging(file_path: String) -> bool {
    let file = match File::create(file_path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Couldn't write to log file, no logs available.");
            return false;
        }
    };
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_ansi(false)
        .with_writer(Mutex::new(file))
        .with_thread_ids(true)
        .finish();
    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("Setting global tracing subscriber failed.");
        return false;
    }
    true
}
