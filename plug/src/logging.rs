use std::{
    ffi::{c_char, CStr},
    fs::File,
    sync::Mutex,
};

use tracing::Level;

#[allow(dead_code)]
#[repr(C)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[no_mangle]
pub extern "C" fn tk_init_logging_stdout(level: LogLevel) -> bool {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level.to_tracing_level())
        .with_ansi(false)
        .with_thread_ids(true)
        .finish();

    if let Err(_) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Setting global tracing subscriber failed.");
        return false;
    }
    return true;
}

#[no_mangle]
pub extern "C" fn tk_init_logging(level: LogLevel, _path: *const c_char) -> bool {
    if _path.is_null() {
        return false;
    }
    let path = unsafe { CStr::from_ptr(_path) }.to_str().unwrap();
    let file = match File::create(path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Couldn't write to log file, no logs available.");
            return false;
        }
    };
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level.to_tracing_level())
        .with_ansi(false)
        .with_writer(Mutex::new(file))
        .with_thread_ids(true)
        .finish();

    if let Err(_) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Setting global tracing subscriber failed.");
        return false;
    }
    return true;
}

impl LogLevel {
    pub fn to_tracing_level(&self) -> Level {
        match self {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}
