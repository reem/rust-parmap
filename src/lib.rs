#![license = "MIT"]
#![deny(missing_docs)]
#![deny(warnings)]

//! Parallel Iterator processing.

use std::sync;

/// An Iterator representing a series of computations
/// being run in parallel.
///
/// The yield order of this Iterator is undefined, and
/// is dependent on the scheduling of the underlying
/// parallel computations.
pub struct ParMap<R> {
    left: uint,
    from: Receiver<R>
}

/// A mixin trait for providing the par_map method.
pub trait Parallel<T: Send> {
    /// Do a computation on every element in the Iterator
    /// in parallel.
    fn par_map<R: Send>(self, mapper: fn(T) -> R) -> ParMap<R>;
}

impl<T: Send, I: Iterator<T>> Parallel<T> for I {
    fn par_map<R: Send>(mut self, mapper: fn(T) -> R) -> ParMap<R> {
        let mut pool = sync::TaskPool::new(8, || { proc(_) {} });

        let mut count = 0;
        let (tx, rx) = channel();
        for x in self {
            let tx = tx.clone();
            count += 1;
            pool.execute(proc(_) {
                tx.send(mapper(x));
            });
        }

        ParMap { left: count, from: rx }
    }
}

impl<R: Send> Iterator<R> for ParMap<R> {
    fn next(&mut self) -> Option<R> {
        if self.left == 0 {
          None
        } else {
            self.left -= 1;
            Some(self.from.recv())
        }
    }
}

