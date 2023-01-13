

pub trait Narrow<T> {
    fn narrow( self, above: T, below: T) -> T;
}

impl Narrow<f32> for f32 {
    fn narrow( self, above: f32, below: f32) -> f32 {
        match self {
            a if a < above => above,
            a if a > below => below,
            a => a
        }
    }
}    

