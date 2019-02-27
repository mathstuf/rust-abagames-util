// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

use std::iter::{self, Chain};
use std::mem;
use std::slice::{Iter, IterMut};

use crates::rayon::iter::Chain as ParChain;
use crates::rayon::prelude::*;
use crates::rayon::slice::Iter as ParIter;
use crates::rayon::slice::IterMut as ParIterMut;

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

const MAX_RECOMMENDED_SIZE: usize = 160_000;

impl<T> Pool<T> {
    fn check_size() {
        if mem::size_of::<T>() > MAX_RECOMMENDED_SIZE {
            eprintln!(
                "Creating a pool with an item size > {} ({}); this is known to have issues",
                MAX_RECOMMENDED_SIZE,
                mem::size_of::<T>(),
            );
        }
    }

    /// Create a new pool with filled with objects created by a function.
    pub fn new<F>(size: usize, ctor: F) -> Self
    where
        F: Fn() -> T,
    {
        assert_ne!(size, 0);
        Self::check_size();

        Pool {
            pool: iter::repeat_with(ctor).take(size).collect(),
            in_use: Vec::with_capacity(size),
        }
    }

    /// Create a new pool with filled with indexed objects created by a function.
    pub fn new_indexed<F>(size: usize, ctor: F) -> Self
    where
        F: Fn(usize) -> T,
    {
        assert_ne!(size, 0);
        Self::check_size();

        Pool {
            pool: (0..size).map(ctor).collect(),
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
        }
        .expect("at least one object should be available")
    }

    /// Clears the pool of all objects.
    pub fn clear(&mut self) {
        self.pool.extend(self.in_use.drain(..))
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
        self.in_use.iter().chain(self.pool.iter())
    }

    #[inline]
    /// An iterator over all objects.
    pub fn iter_all_mut(&mut self) -> Chain<IterMut<T>, IterMut<T>> {
        self.in_use.iter_mut().chain(self.pool.iter_mut())
    }

    /// Run a function for each in-use object and return expired objects to the pool.
    pub fn run<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut T) -> PoolRemoval,
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
    where
        F: FnMut(&mut T, PoolChainIter<T>) -> PoolRemoval,
    {
        let mut idx = 0;
        while idx < self.in_use.len() {
            let status = {
                let (left, right) = self.in_use.split_at_mut(idx);
                let (item, right) = right
                    .split_first_mut()
                    .expect("expected there to be at least one item on the right");
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
    where
        F: Fn(&T) -> PoolRemoval,
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

impl<T> Pool<T>
where
    T: Sync,
{
    #[inline]
    /// A parallel iterator over in-use objects.
    pub fn par_iter(&self) -> ParIter<T> {
        self.in_use.par_iter()
    }

    #[inline]
    /// A parallel iterator over all objects.
    pub fn par_iter_all(&self) -> ParChain<ParIter<T>, ParIter<T>> {
        self.in_use.par_iter().chain(self.pool.par_iter())
    }
}

impl<T> Pool<T>
where
    T: Send,
{
    #[inline]
    /// A parallel iterator over in-use objects.
    pub fn par_iter_mut(&mut self) -> ParIterMut<T> {
        self.in_use.par_iter_mut()
    }

    #[inline]
    /// A parallel iterator over all objects.
    pub fn par_iter_all_mut(&mut self) -> ParChain<ParIterMut<T>, ParIterMut<T>> {
        self.in_use.par_iter_mut().chain(self.pool.par_iter_mut())
    }
}

#[cfg(test)]
mod test {
    use Pool;

    use super::MAX_RECOMMENDED_SIZE;

    #[test]
    fn test_pool_new() {
        let mut pool = Pool::new(10, || 0);
        *pool.get().unwrap() = 1;
        let in_use = pool.iter().collect::<Vec<_>>();
        assert_eq!(in_use.len(), 1);
        assert_eq!(*in_use[0], 1);
    }

    #[test]
    fn test_pool_indexed() {
        let mut pool = Pool::new_indexed(10, |i| i);
        assert_eq!(*pool.get().unwrap(), 9);
        assert_eq!(*pool.get().unwrap(), 8);
    }

    #[test]
    fn test_pool_get_empty() {
        let mut pool = Pool::new(1, || 0);
        assert!(pool.get().is_some());
        assert!(pool.get().is_none());
    }

    #[test]
    fn test_pool_get_forced_empty() {
        let mut pool = Pool::new(1, || 0);
        *pool.get().unwrap() = 1;
        assert_eq!(*pool.get_force(), 1);
    }

    #[test]
    fn test_pool_get_forced_empty_big_type() {
        let mut pool = Pool::new(1, || [0u8; MAX_RECOMMENDED_SIZE]);
        (*pool.get().unwrap())[0] = 1;
        assert_eq!((*pool.get_force())[0], 1);
    }
}
