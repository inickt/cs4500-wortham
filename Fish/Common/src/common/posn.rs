
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Posn {
    pub x: u32,
    pub y: u32,
}

impl Posn {
    pub fn new(x: u32, y: u32) -> Posn {
        Posn { x, y }
    }
}