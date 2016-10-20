// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

extern crate itertools;

use std::slice::Iter;

pub struct Pool<T> {
    pool: Vec<T>,
    in_use: Vec<T>,
}

impl<T> Pool<T> {
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

    pub fn get(&mut self) -> Option<&mut T> {
        self.pop()
            .and_then(move |item| {
                self.in_use.push(item);
                self.in_use.last_mut()
            })
    }

    pub fn get_force(&mut self) -> &mut T {
        if let Some(item) = self.pop() {
            self.in_use.push(item);
            self.in_use.last_mut()
        } else {
            self.in_use.first_mut()
        }.unwrap()
    }

    pub fn clear(&mut self) {
        self.pool
            .extend(self.in_use.drain(..))
    }

    pub fn iter(&self) -> Iter<T> {
        self.in_use.iter()
    }
}
