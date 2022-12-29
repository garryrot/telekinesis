use buttplug::{
    client::ButtplugClient,
    core::connector::ButtplugInProcessClientConnectorBuilder,
    server::{
        device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder,
        ButtplugServerBuilder,
    },
};
use telekinesis::{Telekinesis, TkError};

use std::{
    ffi::{c_float, c_void, CString},
    mem::forget,
};
use tracing::error;

mod logging;
mod telekinesis;
mod tests;
mod tests_int;

#[no_mangle]
pub extern "C" fn tk_connect() -> *mut c_void {
    let tk = Telekinesis::connect_with(async {
        let server = ButtplugServerBuilder::default()
            .comm_manager(BtlePlugCommunicationManagerBuilder::default())
            .finish()?;
        let connector = ButtplugInProcessClientConnectorBuilder::default()
            .server(server)
            .finish();
        let client = ButtplugClient::new("Telekinesis");
        client.connect(connector).await?;
        Ok::<ButtplugClient, TkError>(client)
    });

    match tk {
        Ok(unwrapped) => Box::into_raw(Box::new(unwrapped)) as *mut c_void,
        Err(_) => {
            error!("Failed creating server.");
            std::ptr::null_mut()
        }
    }
}

#[tracing::instrument]
#[no_mangle]
pub extern "C" fn tk_scan_for_devices(_tk: *const c_void) -> bool {
    get_handle_unsafe(_tk).scan_for_devices()
}

#[tracing::instrument]
#[no_mangle]
pub extern "C" fn tk_vibrate_all(_tk: *const c_void, speed: c_float) -> i32 {
    get_handle_unsafe(_tk).vibrate_all(speed)
}

#[tracing::instrument]
#[no_mangle]
pub extern "C" fn tk_await_next_event(_tk: *const c_void) -> *mut i8 {
    assert!(false == _tk.is_null());
    let mut tk = unsafe { Box::from_raw(_tk as *mut Telekinesis) };
    let evt = tk.get_next_event();
    forget(tk);

    // Big bad hack to get it to work without having to do the effort
    // to return and manage a list of cstrings. TBD
    if evt.len() > 0 {
        CString::new(evt[0].clone()).expect("not null").into_raw() as *mut i8
        // just drop if there is more than one event lol
    } else {
        std::ptr::null_mut()
    }
}

#[tracing::instrument]
#[no_mangle]
pub extern "C" fn tk_free_event(_: *const c_void, event: *mut i8) {
    assert!(false == event.is_null());
    unsafe { CString::from_raw(event) }; // dealloc string
}

#[tracing::instrument]
#[no_mangle]
pub extern "C" fn tk_stop_all(_tk: *const c_void) -> i32 {
    get_handle_unsafe(_tk).stop_all()
}

#[tracing::instrument]
#[no_mangle]
pub extern "C" fn tk_close(_tk: *mut c_void) {
    let tk = unsafe { Box::from_raw(_tk as *mut Telekinesis) };
    tk.tk_close();
}

fn get_handle_unsafe(tk: *const c_void) -> &'static Telekinesis {
    assert!(false == tk.is_null());
    unsafe { &*(tk as *const Telekinesis) }
}
