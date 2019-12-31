use num::Num;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Coord<T>
where
    T: Num,
{
    pub x: T,
    pub y: T,
}

impl<T> Coord<T>
where
    T: Num,
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Default for Coord<T>
where
    T: Num + Default,
{
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}
