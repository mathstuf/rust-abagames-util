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

/// An iterator over objects within the pool.
pub type PoolChainIter<'a, T> = Chain<Iter<'a, T>, Iter<'a, T>>;

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

    /// Create a new pool with filled with indexed objects created by a function.
    pub fn new_indexed<F>(size: usize, ctor: F) -> Self
        where F: Fn(usize) -> T,
    {
        assert_ne!(size, 0);

        Pool {
            pool: (0..size)
                .map(ctor)
                .collect(),
            in_use: Vec::with_capacity(size),
        }
    }

    #[inline]
    /// Get an object from the pool.
    fn pop(&mut self) -> Option<T> {
        self.pool.pop()
    }

    /// Get a free object from the pool.
    pub fn get(&mut self) -> Option<&mut T> {
        if let Some(item) = self.pop() {
            self.in_use.push(item);
            self.in_use.last_mut()
        } else {
            None
        }
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

    #[inline]
    /// An iterator over in-use objects.
    pub fn iter(&self) -> Iter<T> {
        self.in_use.iter()
    }

    #[inline]
    /// An iterator over in-use objects.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.in_use.iter_mut()
    }

    #[inline]
    /// An iterator over all objects.
    pub fn iter_all(&self) -> Chain<Iter<T>, Iter<T>> {
        self.in_use.iter()
            .chain(self.pool.iter())
    }

    #[inline]
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
            let status = func(&mut self.in_use[idx]);
            match status {
                PoolRemoval::Remove => self.pool.push(self.in_use.swap_remove(idx)),
                PoolRemoval::Keep => idx += 1,
            }
        }
    }

    /// Run a function for each in-use object with access to the other objects in the pool and
    /// return expired objects to the pool.
    pub fn run_ref<F>(&mut self, mut func: F)
        where F: FnMut(&mut T, PoolChainIter<T>) -> PoolRemoval,
    {
        let mut idx = 0;
        while idx < self.in_use.len() {
            let status = {
                let (left, right) = self.in_use.split_at_mut(idx);
                let (item, right) = right.split_first_mut().expect("expected there to be at least one item on the right");
                func(item, left.iter().chain(right.iter()))
            };
            match status {
                PoolRemoval::Remove => self.pool.push(self.in_use.swap_remove(idx)),
                PoolRemoval::Keep => idx += 1,
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
