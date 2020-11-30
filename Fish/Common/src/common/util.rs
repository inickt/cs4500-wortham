/// This file contains utility functions to abstract over common use cases of built-in
/// Rust functionality.
use std::iter::FromIterator;
use std::time::{ Instant, Duration };

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

/// Return a Vec of all the minimum values in the given iterable,
/// as determined by the key function.
pub fn all_min_by_key<I, T, K, F>(iter: I, mut f: F) -> std::vec::IntoIter<T> where 
    I: Iterator<Item = T>,
    K: Ord + Copy,
    F: FnMut(&T) -> K
{
    let mut results = vec![];
    let mut min_key = None;
    for element in iter {
        let key = f(&element);
        if min_key.map_or(true, |min_key| key < min_key) {
            min_key = Some(key);
            results.clear();
        }
        if min_key.map_or(true, |min_key| key == min_key) {
            results.push(element);
        }
    }
    results.into_iter()
}

/// Return a Vec of all the maximum values in the given iterable,
/// as determined by the key function.
pub fn all_max_by_key<I, T, K, F>(iter: I, mut f: F) -> std::vec::IntoIter<T> where 
    I: Iterator<Item = T>,
    K: Ord + Copy,
    F: FnMut(&T) -> K
{
    let mut results = vec![];
    let mut max_key = None;
    for element in iter {
        let key = f(&element);
        if max_key.map_or(true, |max_key| key > max_key) {
            max_key = Some(key);
            results.clear();
        }
        if max_key.map_or(true, |max_key| key == max_key) {
            results.push(element);
        }
    }
    results.into_iter()
}

/// Keep retrying the given function until it returns a Some(value).
/// If such a value wasn't returned within the given timeout, return None.
///
/// This expects the function to complete in a relatively short time. If
/// the function runs for a long time, try_with_timeout will potentially
/// block for longer than `timeout`
pub fn try_with_timeout<F, U>(timeout: Duration, mut f: F) -> Option<U>
    where F: FnMut() -> Option<U>
{
    let start_time = Instant::now();
    loop {
        match f() {
            Some(value) => return Some(value),
            None if start_time.elapsed() < timeout => continue,
            _ => return None,
        }
    }
}
