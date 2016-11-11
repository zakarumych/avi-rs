use std::io::{self, Read, Seek};
use std::fmt;
use std::mem::size_of;

use riff;
use deser::Deser;
use data::*;

use super::{AVIError, AVIResult, FormatError, UnknownFCC, Format};


#[derive(Clone, Debug)]
pub struct StreamIndex {
    header: StreamIndexHeader,
    entries: Vec<StreamIndexEntry>,
}

impl StreamIndex {
    pub fn entry(&self, id: usize) -> Option<&StreamIndexEntry> {
        self.entries.get(id)
    }

    pub fn from_riff<'a, T: 'a + io::Read + io::Seek + fmt::Debug>(chunk: &mut riff::Chunk<'a, T>) -> AVIResult<Self> {
        let mut read = chunk.read();
        let header: StreamIndexHeader = Deser::deser(&mut read)?;
        if header.longs_per_entry as usize * 4 != size_of::<StreamIndexEntry>() {
            Err(format_error!())
        } else {
            Ok(StreamIndex {
                header: header,
                entries: Deser::deser(&mut read)?
            })
        }
    }
}


#[derive(Clone, Debug)]
pub struct SuperIndex {
    header: SuperIndexHeader,
    entries: Vec<SuperIndexEntry>,
}

impl SuperIndex {
    pub fn entry(&self, id: usize) -> Option<&SuperIndexEntry> {
        self.entries.get(id)
    }

    pub fn from_riff<'a, T: 'a + io::Read + io::Seek + fmt::Debug>(chunk: &mut riff::Chunk<'a, T>) -> AVIResult<Self> {
        let mut read = chunk.read();
        let header: SuperIndexHeader = Deser::deser(&mut read)?;
        if header.longs_per_entry as usize * 4 != size_of::<SuperIndexEntry>() {
            Err(format_error!())
        } else if header.index_sub_type != 0 && header.index_sub_type != AVI_INDEX_2FIELD {
            Err(format_error!())
        } else if header.index_type != AVI_INDEX_OF_INDEXES {
            unimplemented!();
        } else {
            Ok(SuperIndex {
                header: header,
                entries: Deser::deser(&mut read)?
            })
        }
    }
}
