

pub trait Narrow<T> {
    fn narrow(self, above: T, below: T) -> T;
}

impl Narrow<i64> for i64 {
    fn narrow( self, above: i64, below: i64) -> i64 {
        match self {
            a if a < above => above,
            a if a > below => below,
            a => a
        }
    }
}    

