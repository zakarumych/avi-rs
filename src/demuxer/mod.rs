

macro_rules! format_error {
	() => (FormatError { line: line!(), file: file!() })
}

mod stream;
mod index;

use std::io::{self, Read, Seek};
use std::borrow::BorrowMut;

use data::*;
use riff;
use deser::Deser;
use fourcc::FourCC;

pub use self::AVIError::*;
use self::Format::*;
use self::stream::*;

#[derive(Debug)]
pub enum AVIError {
	IOError(io::Error),
	FormatError{ line: u32, file: &'static str },
	UnknownFCC(FourCC),
}

impl From<io::Error> for AVIError {
	fn from(err: io::Error) -> Self { IOError(err) }
}
pub type AVIResult<T> = Result<T, AVIError>;

#[derive(Clone, Debug)]
pub enum Format {
	Video(BitmapInfoHeader),
	Audio(WaveFormat),
}

#[derive(Clone, Debug)]
pub struct Demuxer<'a, T: 'a + Read + Seek> {
	header: MainHeader,
	streams: Vec<RawStream>,
	info: Option<riff::List<'a, T>>,
	movi: riff::List<'a, T>,
	idx1: Option<Vec<IndexEntry>>,
}


impl<'a, T: 'a + Read + Seek> Demuxer<'a, T> {
	pub fn from_riff(data: &'a mut riff::Riff<T>) -> AVIResult<Self> {
		let mut data: riff::List<'a, T> = data.iter().next().ok_or(format_error!())??;
		if data.fourcc() != FCC_AVI { return Err(format_error!()); }
		let mut header: Option<MainHeader> = None;
		let mut streams = vec![];
		let mut info: Option<riff::List<'a, T>> = None;
		let mut movi: Option<riff::List<'a, T>> = None;
		let mut idx1: Option<Vec<IndexEntry>> = None;
		for item in data.iter() {
			let node = item?;
			match node.fourcc() {
				FCC_HDRL => {
					for item in node.list_or(format_error!())?.iter() {
						let mut node = item?;
						match node.fourcc() {
							FCC_AVIH => {
								if header.is_some() {
									return Err(format_error!());
								}
								header = Some(Deser::deser(&mut node.chunk_or(format_error!())?.read())?);
							}
							FCC_STRL => {
								streams.push(RawStream::from_riff(&mut node.list_or(format_error!())?)?);
							}
							FCC_JUNK => {
								continue;
							}
							fcc => {
								return Err(UnknownFCC(fcc));
							}
						}
					}
				}
				FCC_INFO => {
					if info.is_some() {
						return Err(format_error!());
					}
					info = Some(node.list_or(format_error!())?);
				}
				FCC_MOVI => {
					if movi.is_some() {
						return Err(format_error!());
					}
					movi = Some(node.list_or(format_error!())?);
				}
				FCC_IDX1 => {
					if idx1.is_some() {
						return Err(format_error!());
					}
					idx1 = Some(Deser::deser(&mut node.chunk_or(format_error!())?.read())?);
				}
				FCC_JUNK => {
					continue;
				}
				fcc => {
					return Err(UnknownFCC(fcc));
				}
			}
		}
		let header = header.ok_or(format_error!())?;
		if header.streams as usize != streams.len() {
			Err(format_error!())
		} else {
			Ok(Demuxer{
				header: header,
				streams: streams,
				info: info,
				movi: movi.ok_or(format_error!())?,
				idx1: idx1
			})
		}
	}
}



