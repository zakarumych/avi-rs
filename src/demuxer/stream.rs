use std::{io, fmt};

use riff;
use deser::Deser;
use data::*;

use super::{AVIError, AVIResult, FormatError, UnknownFCC, Format};
use super::index::StreamIndex;

#[derive(Clone, Debug)]
pub struct RawStream {
	header: StreamHeader,
	format: Format,
	name: Option<String>,
	index: Option<StreamIndex>
}

impl RawStream {
	pub fn from_riff<'a, T: 'a + io::Read + io::Seek + fmt::Debug>(list: &mut riff::List<'a, T>) -> AVIResult<Self> {
		if list.fourcc() != FCC_STRL { return Err(format_error!()); }
		let list = list.iter();
		let mut header: Option<StreamHeader> = None;
		let mut format: Option<Format> = None;
		let mut name: Option<String> = None;
		let mut index: Option<StreamIndex> = None;
		for item in list {
			let mut chunk = item?.chunk_or(format_error!())?;
			match chunk.fourcc() {
				FCC_STRH => {
					if header.is_some() {
						return Err(format_error!());
					}
					header = Some(Deser::deser(&mut chunk.read())?);
				}
				FCC_STRF => {
					if format.is_some() {
						return Err(format_error!());
					}
					match header.map(|h| h.fcc_type).ok_or(format_error!())? {
						FCC_VIDS => {
							format = Some(Format::Video(Deser::deser(&mut chunk.read())?));
						}
						FCC_AUDS => {
							format = Some(Format::Audio(Deser::deser(&mut chunk.read())?));
						}
						FCC_JUNK => {
							continue;
						}
						fcc => {
							return Err(UnknownFCC(fcc));
						}
					}
				}
				FCC_STRN => {
					if name.is_some() {
						return Err(format_error!());
					}
					name = Some(Deser::deser(&mut chunk.read())?);
				}
				FCC_INDX => {
					if index.is_some() {
						return Err(format_error!());
					}
					index = Some(StreamIndex::from_riff(&mut chunk)?);
				}
				FCC_JUNK => {
					continue;
				}
				fcc => {
					return Err(UnknownFCC(fcc));
				}
			}
		}

		Ok(RawStream {
			header: header.ok_or(format_error!())?,
			format: format.ok_or(format_error!())?,
			name: name,
			index: index
		})
	}
}
