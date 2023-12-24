
#[allow(dead_code)]
pub fn enable_log() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .finish(),
    )
    .unwrap();
}

#[allow(dead_code)]
pub fn enable_trace() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .finish(),
    )
    .unwrap();
}

use tracing::Level;