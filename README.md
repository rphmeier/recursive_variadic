[![Build Status](https://travis-ci.org/rphmeier/recursive_variadic.svg)](https://travis-ci.org/rphmeier/recursive_variadic)

Something like variadic templates or a type map using recursive generics.
The advantage over a hashmap-backed type map is that the monomorphizations of the generic structs and getter functions should compile down to code as efficient as a normal struct field access.
