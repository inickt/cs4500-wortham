use std::iter::FromIterator;

/// Creates a collection of length n with each element mapped from
/// the current element index to f(index)
pub fn make_n<Elem, F, Collection>(n: usize, f: F) -> Collection
    where F: FnMut(usize) -> Elem,
          Collection: FromIterator<Elem>,
{
    (0 .. n).map(f).collect()
}

/// Map a function on each element of a slice, yielding a
/// new Vec in the process.
pub fn map_slice<T, U, F>(slice: &[T], f: F) -> Vec<U>
    where F: FnMut(&T) -> U
{
    slice.iter().map(f).collect()
}