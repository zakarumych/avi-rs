use std::io::{self, Read};
use std::borrow::BorrowMut;
use std::mem::size_of;

use deser::{Deser, PlainOldData};
use fourcc::FourCC;

/*

    Magic numbers

*/

pub const FCC_AVI:  FourCC = FourCC([b'A', b'V', b'I', b' ']);
pub const FCC_HDRL: FourCC = FourCC([b'h', b'd', b'r', b'l']);
pub const FCC_AVIH: FourCC = FourCC([b'a', b'v', b'i', b'h']);
pub const FCC_STRL: FourCC = FourCC([b's', b't', b'r', b'l']);
pub const FCC_STRH: FourCC = FourCC([b's', b't', b'r', b'h']);
pub const FCC_STRF: FourCC = FourCC([b's', b't', b'r', b'f']);
pub const FCC_STRN: FourCC = FourCC([b's', b't', b'r', b'n']);
pub const FCC_STRD: FourCC = FourCC([b's', b't', b'r', b'd']);
pub const FCC_INFO: FourCC = FourCC([b'I', b'N', b'F', b'O']);
pub const FCC_MOVI: FourCC = FourCC([b'm', b'o', b'v', b'i']);
pub const FCC_REC:  FourCC = FourCC([b'r', b'e', b'c', b' ']);
pub const FCC_IDX1: FourCC = FourCC([b'i', b'd', b'x', b'1']);
pub const FCC_INDX: FourCC = FourCC([b'i', b'n', b'd', b'x']);


pub const FCC_VIDS: FourCC = FourCC([b'v', b'i', b'd', b's']);
pub const FCC_AUDS: FourCC = FourCC([b'a', b'u', b'd', b's']);
pub const FCC_TXTS: FourCC = FourCC([b't', b'x', b't', b's']);
pub const FCC_JUNK: FourCC = FourCC([b'J', b'U', b'N', b'K']);


pub const AVI_INDEX_OF_INDEXES: u8 = 0; // Not set
pub const AVI_INDEX_2FIELD: u8 = 2;


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct BITMAPINFOHEADER {
    pub size: u32,
    pub width: i32,
    pub height: i32,
    pub planes: u16,
    pub bit_count: u16,
    pub compression: u32,
    pub size_image: u32,
    pub x_pels_per_meter: i32,
    pub y_pels_per_meter: i32,
    pub clr_used: u32,
    pub clr_important: u32,
}
unsafe impl PlainOldData for BITMAPINFOHEADER {}

pub type BitmapInfoHeader = BITMAPINFOHEADER;


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct WAVEFORMATEX {
    pub format_tag: u16,
    pub channels: u16,
    pub samples_per_sec: u32,
    pub avg_bytes_per_sec: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub size: u16,
}
unsafe impl PlainOldData for WAVEFORMATEX {}

#[derive(Clone, Debug)]
pub struct WaveFormat {
    header: WAVEFORMATEX,
    extra: Vec<u8>,
}


impl Deser for WaveFormat {
    fn deser<R: Read>(read: &mut R) -> io::Result<WaveFormat> {
        let read = read.borrow_mut();
        let header = WAVEFORMATEX::deser(read)?;
        let mut extra = vec![0; header.size as usize];
        read.read_exact(&mut extra[..])?;
        Ok(WaveFormat { header: header, extra: extra })
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MainHeader {
    pub micro_sec_per_frame: u32,
    pub max_bytes_per_sec: u32,
    pub padding_granularity: u32,
    pub flags: u32,
    pub total_frames: u32,
    pub initial_frames: u32,
    pub streams: u32,
    pub suggested_buffer_suze: u32,
    pub width: u32,
    pub height: u32,
    pub reserved: [u32; 4],
}
unsafe impl PlainOldData for MainHeader {}

#[derive(Copy, Clone, Debug)]
pub enum Flag {
    AVIF_HASINDEX,         // The file has an index 9 
    AVIF_MUSTUSEINDEX,     // The order in which the video and audio chunks must b e replayed is determined by the index and may differ from the order in which those chunks o ccur in the file. 
    AVIF_ISINTERLEAVED,    // The streams are prop erly interleaved into each other 
    AVIF_WASCAPTUREFILE,   // The file was captured. The interleave might b e weird. 
    AVIF_COPYRIGHTED,      // Ignore it 
    AVIF_TRUSTCKTYPE,      // (Op en-DML only!) This flag indicates that the keyframe flags in the index are reliable. If this flag is not set in an Op en-DML file, the keyframe flags could b e defective without technically rendering the file invali
}


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct RECT {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}
unsafe impl PlainOldData for RECT {}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct StreamHeader {
    pub fcc_type: FourCC,
    pub fcc_handler: FourCC,
    pub flags: u32,
    pub priority: u16,
    pub language: u16,
    pub initial_frams: u32,
    pub scale: u32,
    pub rate: u32,
    pub start: u32,
    pub length: u32,
    pub suggested_buffer_suze: u32,
    pub quality: u32,
    pub sample_size: u32,
    pub frame: RECT,
}
unsafe impl PlainOldData for StreamHeader {}

#[derive(Copy, Clone, Debug)]
pub enum StreamFlag {
    AVISF_DISABLED,             // Stream should not b e activated by default
    AVISF_VIDEO_PALCHANGES,     // Stream is a video stream using palettes where the palette is changing during playback.
}


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct IndexEntry {
    pub ckid: u32,
    pub flags: u32,
    pub chunk_offset: u32,
    pub chunk_length: u32,
}
unsafe impl PlainOldData for IndexEntry {}

#[derive(Copy, Clone, Debug)]
pub enum IndexFlags {
    AVIIF_KEYFRAME,     //The chunk the entry refers to is a keyframe.
    AVIIF_LIST,         //The entry p oints to a list, not to a chunk. 
    AVIIF_FIRSTPART,    //Indicates this chunk needs the frames following it to b e used; it cannot stand alone.
    AVIIF_LASTPART,     //Indicates this chunk needs the frames preceding it to b e used; it cannot stand alone. 
    AVIIF_NOTIME,       //The duration which is applied to the corresp onding chunk is 0
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct StreamIndexHeader {
    pub fcc: FourCC,
    pub cb: u32,
    pub longs_per_entry: u16,
    pub index_sub_type: u8,
    pub index_type: u8,
    pub entries_in_use: u32,
    pub chunk_id: u32,
    pub reserved: [u32;3],
}
unsafe impl PlainOldData for StreamIndexHeader {}


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct StreamIndexEntry { adw: u32 }
unsafe impl PlainOldData for StreamIndexEntry {}



#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct SuperIndexHeader {
    pub fcc: FourCC,
    pub cb: u32,
    pub longs_per_entry: u16,
    pub index_sub_type: u8,
    pub index_type: u8,
    pub entries_in_use: u32,
    pub chunk_id: u32,
    pub reserved: [u32;3],
}
unsafe impl PlainOldData for SuperIndexHeader {}


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct SuperIndexEntry { offset: u64, size: u32, duration: u32 }
unsafe impl PlainOldData for SuperIndexEntry {}
