//! This is an implementation of variadic templates using recursive generics.
//! It functions something like a TypeMap, but the optimizer should make getting
//! and adding values much faster.

use std::any::{Any, TypeId};
use std::mem;

/// The building block trait for recursive variadics.
pub trait RecursiveVariadic {
    /// Try to get the field of type N.
    fn get<N: Any>(&self) -> Option<&N>;
    /// Try to get the field of type N mutably.
    fn get_mut<N: Any>(&mut self) -> Option<&mut N>;
    /// Add a field of type N to this.
    fn and<N: Any>(self, val: N) -> Entry<N, Self> where Self: Sized {
        Entry {
            data: val,
            parent: self,
        }
    }
}

/// The base case for recursive variadics: no fields.
pub type Empty = ();
impl RecursiveVariadic for Empty {
    fn get<N: Any>(&self) -> Option<&N> { None }
    fn get_mut<N: Any>(&mut self) -> Option<&mut N> { None }
}

/// Wraps some field data and a parent, which is either another Entry or Empty
pub struct Entry<T, R> {
    data: T,
    parent: R,
}

impl<T: Any, R: RecursiveVariadic> RecursiveVariadic for Entry<T, R> {
    fn get<N: Any>(&self) -> Option<&N> { 
        if TypeId::of::<N>() == TypeId::of::<T>() {
            Some(unsafe { mem::transmute(&self.data) })
        } else {
            self.parent.get()
        }
    }
    fn get_mut<N: Any>(&mut self) -> Option<&mut N> { 
        if TypeId::of::<N>() == TypeId::of::<T>() {
            Some(unsafe { mem::transmute(&mut self.data) })
        } else {
            self.parent.get_mut()
        }
    }
}

/// This macro is a standin for the way this would be implemented 
/// with Higher Kinded Types. It creates a wrapper struct that for every N
/// puts an entry in the RecursiveVariadic it wraps for a Output<N>.
///
/// # Examples
///
/// This example creates a wrapper that gets a thing.
///
/// ```rust
///     #[macro_use]
///     extern crate recursive_variadic;
///
///     use recursive_variadic::*;
///
///     create_wrapper!(VecHolder, N => Vec<N>);
/// 
///     fn vec_variadic() {
///         // this struct maps types to a vector of that type.
///         let mut my_vecs = VecHolder::new()
///             .and_default::<usize>()
///             .and_default::<bool>()
///             .and::<u32>(Vec::new()); // if you want to provide a value
///         my_vecs.get_mut::<usize>().unwrap().push(2);
///         my_vecs.get_mut::<bool>().unwrap().push(false);
///         my_vecs.get_mut::<u32>().unwrap().push(0);
/// 
///         assert!(my_vecs.get::<String>().is_none());
///     }
///
///     fn main() { vec_variadic() }
/// ```
///
/// Using an creating a wrapper around an Id type is redundant,
/// but better shows what this macro does.
///
/// ```rust
///     #[macro_use]
///     extern crate recursive_variadic;
///
///     use recursive_variadic::*;
///
///     type Id<N> = N;
/// 
///     fn id_variadic() {
///         create_wrapper!(IdVariadic, N => Id<N>, PRIV);
///         let mut my_things = IdVariadic::new()
///             .and_default::<usize>()
///             .and(23i32);
/// 
///         *my_things.get_mut::<i32>().unwrap() += 1;
///         assert!(my_things.get::<usize>().is_some());
///     }
///     
///     fn main() { id_variadic() }
/// ```

#[macro_export]
macro_rules! create_wrapper {
    ($name:ident, N => $output:ident<N>) => {
        create_wrapper!($name, $output, pub);
    };

    ($name:ident, N => $output:ident<N>, PRIV) => {
        create_wrapper!($name, $output, );
    };

    ($name:ident, $output:ident, $($vis:tt)*) => {
        use std::any::Any;
        create_wrapper!(
            $($vis)* struct $name<R> {
                data: R,
            }
        );

        create_wrapper!(
            impl $name<()> {
                $($vis)* fn new() -> Self {
                    $name {
                        data: ()
                    }
                }
            }
        );

        create_wrapper!(
            impl<R: RecursiveVariadic> $name<R> {
                $($vis)* fn get<N>(&self) -> Option<&$output<N>>
                where $output<N>: Any {
                    self.data.get()
                }

                $($vis)* fn get_mut<N>(&mut self) -> Option<&mut $output<N>>
                where $output<N>: Any {
                    self.data.get_mut()
                }

                $($vis)* fn and<N: Any>(self, val: $output<N>) -> $name<Entry<$output<N>, R>>
                where $output<N>: Any {
                    $name {
                        data: self.data.and(val)
                    }
                }

                $($vis)* fn and_default<N: Any>(self) -> $name<Entry<$output<N>, R>>
                where $output<N>: Any + Default {
                    $name {
                        data: self.data.and(Default::default())
                    }
                }
            }
        );
    };

    ($i:item) => {$i};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let thing = ().and(23).and(0usize).and("Hello!");
        assert!(thing.get::<i32>().is_some());
        assert!(thing.get::<usize>().is_some());
        assert!(thing.get::<&'static str>().is_some());
        assert!(thing.get::<bool>().is_none());
    }

    create_wrapper!(VecHolder, N => Vec<N>);

    #[test]
    fn vec_variadic() {
        // this struct will hold a vector of usize, bool, and u32.
        let mut my_vecs = VecHolder::new()
            .and_default::<usize>()
            .and_default::<bool>()
            .and::<u32>(Vec::new()); // just to prove they're the same.
        my_vecs.get_mut::<usize>().unwrap().push(2);
        my_vecs.get_mut::<bool>().unwrap().push(false);
        my_vecs.get_mut::<u32>().unwrap().push(0);

        assert!(my_vecs.get::<String>().is_none());
    }
}