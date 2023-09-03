
pub trait Narrow<T> {
    fn narrow(self, above: T, below: T) -> T;
}

impl<T: PartialOrd> Narrow<T> for T
{
    fn narrow(self, above: T, below: T) -> T {
         match self {
             a if a < above => above,
             a if a > below => below,
             a => a
         }
     }
}    

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

macro_rules! assert_timeout {
    ($cond:expr, $arg:tt) => {
        // starting time
        let start: Instant = Instant::now();
        while !$cond {
            thread::sleep(Duration::from_millis(10));
            if start.elapsed().as_secs() > 5 {
                panic!($arg);
            }
        }
    };
}

pub(crate) use assert_timeout;
use tracing::Level;