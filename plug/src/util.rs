

pub trait Narrow<T> {
    fn narrow( self, above: T, below: T) -> T;
}

impl Narrow<f64> for f64 {
    fn narrow( self, above: f64, below: f64) -> f64 {
        match self {
            a if a < above => above,
            a if a > below => below,
            a => a
        }
    }
}    

