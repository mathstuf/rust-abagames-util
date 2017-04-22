// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying file LICENSE for details.

use crates::itertools;

use std::iter::Chain;
use std::slice::{Iter, IterMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Whether to keep or remove a pool entity after stepping it.
pub enum PoolRemoval {
    /// Keep the entity in the pool.
    Keep,
    /// Remove the entity from the pool.
    Remove,
}

/// An entity pool of a fixed size.
pub struct Pool<T> {
    /// The unused objects.
    pool: Vec<T>,
    /// The in-use objects.
    in_use: Vec<T>,
}

impl<T> Pool<T> {
    /// Create a new pool with filled with objects created by a function.
    pub fn new<F>(size: usize, ctor: F) -> Self
        where F: Fn() -> T,
    {
        assert_ne!(size, 0);

        Pool {
            pool: itertools::repeat_call(ctor)
                .take(size)
                .collect(),
            in_use: Vec::with_capacity(size),
        }
    }

    /// Get an object from the pool.
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
        }.expect("at least one object should be available")
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

    /// An iterator over in-use objects.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.in_use.iter_mut()
    }

    /// An iterator over all objects.
    pub fn iter_all(&self) -> Chain<Iter<T>, Iter<T>> {
        self.in_use.iter()
            .chain(self.pool.iter())
    }

    /// An iterator over all objects.
    pub fn iter_all_mut(&mut self) -> Chain<IterMut<T>, IterMut<T>> {
        self.in_use.iter_mut()
            .chain(self.pool.iter_mut())
    }

    /// Run a function for each in-use object and return expired objects to the pool.
    pub fn run<F>(&mut self, mut func: F)
        where F: FnMut(&mut T) -> PoolRemoval,
    {
        let mut idx = 0;
        while idx < self.in_use.len() {
            if func(&mut self.in_use[idx]) == PoolRemoval::Remove {
                let item = self.in_use.swap_remove(idx);
                self.pool.push(item);
            } else {
                idx += 1;
            }
        }
    }

    /// Expire objects which may be returned to the pool.
    pub fn expire<F>(&mut self, pred: F)
        where F: Fn(&T) -> PoolRemoval,
    {
        let mut idx = 0;
        while idx < self.in_use.len() {
            if pred(&self.in_use[idx]) == PoolRemoval::Remove {
                let item = self.in_use.swap_remove(idx);
                self.pool.push(item);
            } else {
                idx += 1;
            }
        }
    }
}
