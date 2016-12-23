// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate itertools;

use std::slice::Iter;

/// An entity pool of a fixed size.
pub struct Pool<T> {
    pool: Vec<T>,
    in_use: Vec<T>,
}

impl<T> Pool<T> {
    /// Create a new pool with filled with objects created by a function.
    pub fn new<F>(size: usize, ctor: F) -> Self
        where F: Fn() -> T,
    {
        Pool {
            pool: itertools::repeat_call(ctor)
                .take(size)
                .collect(),
            in_use: Vec::with_capacity(size),
        }
    }

    fn pop(&mut self) -> Option<T> {
        self.pool.pop()
    }

    /// Get a free object from the pool.
    pub fn get(&mut self) -> Option<&mut T> {
        self.pop()
            .and_then(move |item| {
                self.in_use.push(item);
                self.in_use.last_mut()
            })
    }

    /// Get an object from the pool.
    ///
    /// Removes the oldest in-use object if there are no free objects.
    pub fn get_force(&mut self) -> &mut T {
        if let Some(item) = self.pop() {
            self.in_use.push(item);
            self.in_use.last_mut()
        } else {
            self.in_use.first_mut()
        }.unwrap()
    }

    /// Clears the pool of all objects.
    pub fn clear(&mut self) {
        self.pool
            .extend(self.in_use.drain(..))
    }

    /// An iterator over in-use objects.
    pub fn iter(&self) -> Iter<T> {
        self.in_use.iter()
    }
}
