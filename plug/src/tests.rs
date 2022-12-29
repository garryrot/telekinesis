#[cfg(test)]
mod tests {
    use crate::logging::{tk_init_logging, LogLevel};
    use crate::*;
    use std::ptr::null;

    #[test]
    fn init_logging_handles_nullpointr_without_panic() {
        tk_init_logging(LogLevel::Trace, null());
    }

    #[test]
    fn connect_and_scan() {
        let tk = tk_connect();
        tk_scan_for_devices(tk);
    }
}
