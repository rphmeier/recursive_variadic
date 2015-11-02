//! This is an implementation of variadic templates using recursive generics.
//! It functions something like a TypeMap, but the optimizer should make getting
//! and adding values much faster.

use std::any::{Any, TypeId};
use std::mem;

pub trait Key {
    type Value: Any;
}

/// The building block trait for recursive variadics.
pub trait RecursiveVariadic {
    /// Try to get the value for N.
    fn get<N: Key>(&self) -> Option<&N::Value>;
    /// Try to get the value for N mutably.
    fn get_mut<N: Key>(&mut self) -> Option<&mut N::Value>;
    /// Add a key-value pair to this.
    fn and<N: Key>(self, val: N::Value) -> Entry<N, Self> where Self: Sized {
        Entry {
            data: val,
            parent: self,
        }
    }
    /// Add the default value for N
    fn and_default<N: Key>(self) -> Entry<N, Self> 
    where N::Value: Default, Self: Sized {
        self.and(N::Value::default())
    }
}

/// The base case for recursive variadics: no fields.
pub type Empty = ();
impl RecursiveVariadic for Empty {
    fn get<N: Key>(&self) -> Option<&N::Value> { None }
    fn get_mut<N: Key>(&mut self) -> Option<&mut N::Value> { None }
}

/// Wraps some field data and a parent, which is either another Entry or Empty
pub struct Entry<T: Key, R> {
    data: T::Value,
    parent: R,
}

impl<T: Key, R: RecursiveVariadic> RecursiveVariadic for Entry<T, R> {
    fn get<N: Key>(&self) -> Option<&N::Value> { 
        if TypeId::of::<N::Value>() == TypeId::of::<T::Value>() {
            Some(unsafe { mem::transmute(&self.data) })
        } else {
            self.parent.get::<N>()
        }
    }
    fn get_mut<N: Key>(&mut self) -> Option<&mut N::Value> { 
        if TypeId::of::<N::Value>() == TypeId::of::<T::Value>() {
            Some(unsafe { mem::transmute(&mut self.data) })
        } else {
            self.parent.get_mut::<N>()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;

    #[test]
    fn it_works() {
        impl Key for i32 { type Value = i32; }
        impl Key for usize { type Value = Vec<usize>; }
        impl Key for bool { type Value = bool; }
        impl Key for &'static str { type Value = &'static str; }

        let mut thing = ().and::<i32>(23).and_default::<usize>().and::<&'static str>("Hello!");
        thing.get_mut::<usize>().unwrap().push(1);
        assert!(thing.get::<i32>().is_some());
        assert!(thing.get::<&'static str>().is_some());
        assert!(thing.get::<bool>().is_none());
    }
}