use std::borrow::BorrowMut;
use std::io::{self, Read};

pub trait Deser: Sized {
	fn deser<R: Read+::std::fmt::Debug>(read: &mut R) -> io::Result<Self>;
}

pub unsafe trait PlainOldData: Copy + Sized {}

impl<T> Deser for T where T: PlainOldData {
    fn deser<R: Read+::std::fmt::Debug>(read: &mut R) -> io::Result<T> {
    	let mut read = read.borrow_mut();
        unsafe {
            let mut value = ::std::mem::uninitialized();
            let buf = ::std::slice::from_raw_parts_mut(&mut value as *mut Self as *mut u8,
                                                       ::std::mem::size_of::<Self>());
            match read.read_exact(buf) {
                Ok(()) => Ok(value),
                Err(err) => {
                    ::std::mem::forget(value);
                    println!("Failed to parse {} from {:?} cause of {}", ::typename::<T>(), read, err);
                    Err(err)
                }
            }
        }
    }
}

impl<T> Deser for Vec<T> where T: Deser {
	fn deser<R: Read+::std::fmt::Debug>(read: &mut R) -> io::Result<Vec<T>> {
		let mut result = vec![];
		loop {
			match T::deser(read) {
				Ok(value) => result.push(value),
				Err(err) => {
					if err.kind() == io::ErrorKind::UnexpectedEof {
						break;
					}
                    println!("Failed to parse Vec<{}> from {:?} cause of {}", ::typename::<T>(), read, err);
					return Err(err);
				}
			}
		}
		Ok(result)
	}
}

impl Deser for String {
	fn deser<R: Read+::std::fmt::Debug>(read: &mut R) -> io::Result<String> {
    	let mut read = read.borrow_mut();
		let mut result = String::new();
		match read.read_to_string(&mut result) {
			Ok(_) => Ok(result),
			Err(err) => {
				println!("Failed to parse String from {:?} cause of {}", read, err);
				Err(err)
			}
		}
	}
}
