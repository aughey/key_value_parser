//! This crate provides a simple key value parser.
//!
//! # Details
//!
//! This parser is designed to be simple and fast. It is not designed to be a full featured parser.
//!
//! If you're wanting to do config file parsing, please choose a more full featured parser and use an
//! established format like JSON, TOML, or YAML.
//!
//! # Design Concerns
//!
//! This parser will allocate and duplicate a lot of data that is contained in the original string slice
//! that was provided to it.  An optimization for a more zero-copy approach is to store string slices
//! in the provided map, rather than allocating new strings.
//!
//! This optimization would require the map to be constrained to the lifetime of the original string slice.
//! There is a special case of string values where when the value is quoted with escape character that we have
//! to allocate a new string.  This is because we have to unescape the string, and we can't do that in place.
//!
//! To handle this, we need a special string enumeration that could container either a string reference or an
//! owned string.
//!
//! # Example
//!
//! For this sort of string:
//! ```pre
//! one=aaaaaaaaaaaa two=bbbbbbbbbbbbbbbb three=ccccccccccc four=dddddddddd
//! ```
//! We could store 8 string slices.  Four of those slices would point to the keys (one, two, three, four),
//! and the other four would point to the values (aaaaaaaaaaaa, bbbbbbbbbbbbbbbb, ccccccccccc, ddddddddd).
//!
//! A slice, is conceptually a pointer to the start of the string, and a length.  Maybe more verbose (but not)
//! implementation accurate, would be that it stores the original string, a index to the start of the slice,
//! and a length.
//!
//! ```pre
//! struct two {
//!   string: something_pointer,
//!   start: 17,
//!   len: 16
//! }
//! ```
//!
//! Ideally, we'd like to have HashMap<&str,&str> where the only allocation of memory is the HashMap itself and the
//! internal data structures it uses to store data.  But our original string contains the "data", so a string length
//! of 100gb would be stored in the hashmap in just a few bytes of data.  This is the zero-copy approach.

pub mod almost_zero_copy;
pub mod full_almost_zero_copy;
pub mod full_copy;
pub mod zero_copy;
pub mod zero_parse;