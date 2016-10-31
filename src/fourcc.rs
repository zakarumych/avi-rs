use std::fmt::{self, Formatter, Display, Debug};
use std::str::from_utf8;
use std::{io, mem};

use deser::PlainOldData;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FourCC(pub [u8; 4]);

impl FourCC {
    pub fn from_array(arr: [u8; 4]) -> Self {
        FourCC(arr)
    }
    pub fn from_ref<'a>(arr: &'a [u8; 4]) -> &'a Self {
        unsafe { mem::transmute(arr) }
    }
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 4 {
            None
        } else {
            Some(Self::from_array([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }
    }
    pub fn from_str(string: &str) -> Option<Self> {
        Self::from_slice(string.as_bytes())
    }
}

impl Display for FourCC {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match from_utf8(&self.0) {
            Ok(string) => f.write_str(string),
            Err(_) => write!(f, "{:?}", self.0),
        }
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match from_utf8(&self.0) {
            Ok(string) => write!(f, "{:?}<{}>", self.0, string),
            Err(_) => write!(f, "{:?}", self.0),
        }
    }
}

unsafe impl PlainOldData for FourCC {}
