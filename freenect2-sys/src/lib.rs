//! # libfreenect2-sys
//!
//! Unsafe C-style bindings to [libfreenect2](https://github.com/OpenKinect/libfreenect2).

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
