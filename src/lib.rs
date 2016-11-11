#![feature(question_mark)]
#![feature(specialization)]
#![feature(core_intrinsics)]
#![feature(zero_one)]

fn typename<T>() -> &'static str {
	unsafe { ::std::intrinsics::type_name::<T>() }
}

macro_rules! inspect_err {
	($result:expr, $format:expr, $($add:expr),*) => ( $result.map_err(|err| { println!($format, $($add),*, err); err }) )
}

extern crate byteorder;

pub mod riff;
pub mod fourcc;
pub mod demuxer;

mod deser;
mod data;
