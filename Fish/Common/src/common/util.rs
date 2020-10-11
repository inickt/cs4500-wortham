use std::iter::FromIterator;

pub fn make_n<Elem, F, Collection>(n: usize, f: F) -> Collection
    where F: FnMut(usize) -> Elem,
          Collection: FromIterator<Elem>,
{
    (0 .. n).map(f).collect()
}