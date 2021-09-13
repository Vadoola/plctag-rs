// plctag-rs
//
// a rust wrapper of libplctag, with rust style APIs and useful extensions.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

#[cfg(feature = "event")]
extern crate dyn_clone;
#[cfg(feature = "event")]
extern crate once_cell;
#[cfg(feature = "event")]
extern crate parking_lot;
extern crate plctag_sys;

pub mod ffi {
    pub use plctag_sys::*;
}

pub mod builder;
mod debug;
#[cfg(feature = "event")]
pub mod event;
mod raw;
mod status;
#[cfg(feature = "value")]
mod value;

pub type Result<T> = std::result::Result<T, Status>;
pub use raw::{RawTag, TagId};
pub use status::Status;

#[cfg(feature = "value")]
pub use value::{Decode, Encode};