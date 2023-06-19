

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