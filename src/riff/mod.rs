use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read, Seek, SeekFrom};
use std::cmp;
use std::mem;
use std::rc::Rc;
use std::cell::RefCell;

use byteorder::{ReadBytesExt, LittleEndian, BigEndian};

use fourcc::FourCC;
use deser::Deser;

pub const LIST: FourCC = FourCC([b'L', b'I', b'S', b'T']);
pub const RIFF: FourCC = FourCC([b'R', b'I', b'F', b'F']);

fn round2up<T: ::std::convert::Into<i64>>(value: T) -> i64 {
    let value = value.into();
    value + 1 - (value + 1) % 2
}


#[derive(Clone)]
struct IOBuffer<'a, T>
    where T: 'a + Read + Seek + Debug
{
    start: u64,
    pos: u64,
    size: u64,
    inner: &'a RefCell<T>,
}

impl<'a, T> IOBuffer<'a, T>
    where T: 'a + Read + Seek + Debug
{
    fn new(iobuff: &'a RefCell<T>, start: u64, size: u64) -> Self {
        IOBuffer {
            start: start,
            pos: 0,
            size: size,
            inner: iobuff,
        }
    }
    fn amount_left(&self) -> u64 {
        self.size - cmp::min(self.pos, self.size)
    }
    fn take_slice(&self, size: u64) -> io::Result<Self> {
        if size > self.amount_left() {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("Unexpected end of {:?} when taking slice {}", self, size)))
        } else {
            Ok(IOBuffer {
                start: self.start + self.pos,
                pos: 0,
                size: size,
                inner: self.inner,
            })
        }
    }
}

impl<'a, T> Read for IOBuffer<'a, T>
    where T: 'a + Read + Seek + Debug
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let toread = cmp::min(buf.len() as u64, self.amount_left()) as usize;
        if toread == 0 {
            Ok(0)
        } else {
            let mut inner = self.inner.borrow_mut();
            inner.seek(SeekFrom::Start(self.start + self.pos))?;
            let read = inner.read(&mut buf[..toread])?;
            self.pos += read as u64;
            Ok(read)
        }
    }
}

impl<'a, T> Seek for IOBuffer<'a, T>
    where T: 'a + Read + Seek + Debug
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let pos = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::Current(pos) if pos >= 0 => self.pos + pos as u64,
            SeekFrom::Current(pos) => {
                let pos = (-pos) as u64;
                if pos > self.pos {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                              "invalid seek to a negative position"));
                } else {
                    self.pos - pos
                }
            }
            SeekFrom::End(pos) if pos >= 0 => self.size + pos as u64,
            SeekFrom::End(pos) => {
                let pos = (-pos) as u64;
                if pos > self.size {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                              "invalid seek to a negative position"));
                } else {
                    self.size - pos
                }
            }
        };
        self.pos = pos;
        Ok(pos)
    }
}


impl<'a, T: 'a + Read + Seek + Debug> Debug for IOBuffer<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "IOBuffer {{ start: {:?}, pos: {:?}, size: {:?} }}", self.start, self.pos, self.size)
    }
}


impl<'a, T: 'a + Read + Seek + Debug> Drop for IOBuffer<'a, T>
    where T: 'a + Read + Seek + Debug
{
    fn drop(&mut self) {
        self.inner.borrow_mut().seek(SeekFrom::Start(self.start + self.pos));
    }
}

//

#[derive(Debug, Clone)]
pub enum Node<'a, T>
    where T: 'a + Read + Seek + Debug
{
    List(List<'a, T>),
    Chunk(Chunk<'a, T>),
}

impl<'a, T> Node<'a, T>
    where T: 'a + Read + Seek + Debug
{
    pub fn fourcc(&self) -> FourCC {
        match *self {
            Node::List(ref item) => item.fourcc(),
            Node::Chunk(ref item) => item.fourcc(),
        }
    }
    pub fn chunk_or<E>(self, error: E) -> Result<Chunk<'a, T>, E> {
        self.chunk_or_else(|_| error)
    }
    pub fn chunk_or_else<E, F>(self, f: F) -> Result<Chunk<'a, T>, E> where F: FnOnce(List<'a, T>) -> E {
        match self {
            Node::Chunk(chunk) => Ok(chunk),
            Node::List(list) => Err(f(list))
        }
    }
    pub fn list_or<E>(self, error: E) -> Result<List<'a, T>, E> {
        self.list_or_else(|_| error)
    }
    pub fn list_or_else<E, F>(self, f: F) -> Result<List<'a, T>, E> where F: FnOnce(Chunk<'a, T>) -> E {
        match self {
            Node::List(list) => Ok(list),
            Node::Chunk(chunk) => Err(f(chunk))
        }
    }
}

//

#[derive(Debug, Clone)]
pub struct Riff<T>
    where T: Read + Seek + Debug
{
    size: u64,
    stream: RefCell<T>,
}

impl<T> Riff<T>
    where T: Read + Seek + Debug
{
    pub fn new(mut stream: T) -> io::Result<Self> {
        let size = stream.seek(io::SeekFrom::End(0))?;
        Ok(Riff {
            size: size,
            stream: RefCell::new(stream),
        })
    }

    pub fn iter<'a>(&'a mut self) -> RiffIter<'a, T> {
        RiffIter { iobuff: IOBuffer::new(&self.stream, 0, self.size) }
    }

    pub fn release(self) -> T {
        self.stream.into_inner()
    }
}

pub struct RiffIter<'a, T>
    where T: 'a + Read + Seek + Debug
{
    iobuff: IOBuffer<'a, T>,
}

impl<'a, T> RiffIter<'a, T>
    where T: 'a + Read + Seek + Debug
{
    fn read_next(&mut self) -> Option<io::Result<List<'a, T>>> {
        if self.iobuff.amount_left() < mem::size_of::<(FourCC, u32, FourCC)>() as u64 {
            return None;
        }
        let fcc = FourCC::deser(&mut self.iobuff);
        match fcc {
            Ok(fcc) => {
                match fcc {
                    RIFF => Some(self.read_next_riff()),
                    _ => self.iobuff.seek(io::SeekFrom::Current(-4)).err().map(Err),
                }
            }
            Err(ref err) if err.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(err) => Some(Err(err)),
        }
    }

    fn read_next_riff(&mut self) -> io::Result<List<'a, T>> {
        let size = self.iobuff.read_u32::<LittleEndian>()? - 4;
        let fcc = FourCC::deser(&mut self.iobuff)?;
        let slice = self.take_stream_slice(size as u64)?;
        self.iobuff.seek(io::SeekFrom::Current(round2up(size) as i64))?;
        Ok(List {
            fcc: fcc,
            iobuff: slice,
        })
    }

    fn take_stream_slice(&self, size: u64) -> io::Result<IOBuffer<'a, T>> {
        self.iobuff.take_slice(size)
    }
}

impl<'a, T> Iterator for RiffIter<'a, T>
    where T: 'a + Read + Seek + Debug
{
    type Item = io::Result<List<'a, T>>;

    fn next(&mut self) -> Option<io::Result<List<'a, T>>> {
        self.read_next()
    }
}


//

#[derive(Debug, Clone)]
pub struct List<'a, T>
    where T: 'a + Read + Seek + Debug
{
    fcc: FourCC,
    iobuff: IOBuffer<'a, T>,
}


impl<'a, T> List<'a, T>
    where T: 'a + Read + Seek + Debug
{
    pub fn iter<'b>(&'b mut self) -> ListIter<'a, 'b, T>
        where 'a: 'b
    {
        self.iobuff.seek(io::SeekFrom::Start(0)).unwrap();
        ListIter { inner: self }
    }
    pub fn fourcc(&self) -> FourCC {
        self.fcc
    }
    pub fn size(&self) -> u64 {
        self.iobuff.size
    }
    fn read_next(&mut self) -> Option<io::Result<Node<'a, T>>> {
        if self.iobuff.amount_left() < mem::size_of::<FourCC>() as u64 {
            return None;
        }

        match FourCC::deser(&mut self.iobuff) {
            Ok(LIST) => {
                if self.iobuff.amount_left() < mem::size_of::<(FourCC, u32)>() as u64 {
                    return None;
                }
                Some(self.read_list().map(Node::List))
            }
            Ok(fcc) => {
                if self.iobuff.amount_left() < mem::size_of::<u32>() as u64 {
                    return None;
                }
                Some(self.read_chunk(fcc).map(Node::Chunk))
            }
            Err(err) => Some(Err(err))
        }
    }
    fn read_list(&mut self) -> io::Result<List<'a, T>> {
        let size = self.iobuff.read_u32::<LittleEndian>()? - 4;
        let fcc = FourCC::deser(&mut self.iobuff)?;
        let slice = self.take_stream_slice(size as u64)?;
        self.iobuff.seek(io::SeekFrom::Current(round2up(size) as i64))?;
        Ok(List {
            fcc: fcc,
            iobuff: slice,
        })
    }

    fn read_chunk(&mut self, fcc: FourCC) -> io::Result<Chunk<'a, T>> {
        let mut size = self.iobuff.read_u32::<LittleEndian>()?;
        if size as u64 > self.iobuff.amount_left() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("Chunk is too big. Chunk: {}. Remaining size: {}", size, self.iobuff.amount_left())));
        }
        let slice = self.take_stream_slice(size as u64)?;
        self.iobuff.seek(io::SeekFrom::Current(round2up(size) as i64))?;
        Ok(Chunk {
            fcc: fcc,
            iobuff: slice,
        })
    }
    fn take_stream_slice(&self, size: u64) -> io::Result<IOBuffer<'a, T>> {
        self.iobuff.take_slice(size)
    }
}

#[derive(Debug)]
pub struct ListIter<'a, 'b, T>
    where T: 'a + Read + Seek + Debug,
          'a: 'b
{
    inner: &'b mut List<'a, T>,
}

impl<'a, 'b, T> Iterator for ListIter<'a, 'b, T>
    where T: 'a + Read + Seek + Debug,
          'a: 'b
{
    type Item = io::Result<Node<'a, T>>;

    fn next(&mut self) -> Option<io::Result<Node<'a, T>>> {
        self.inner.read_next()
    }
}


//

#[derive(Debug, Clone)]
pub struct Chunk<'a, T>
    where T: 'a + Read + Seek + Debug
{
    fcc: FourCC,
    iobuff: IOBuffer<'a, T>,
}

impl<'a, T> Chunk<'a, T>
    where T: 'a + Read + Seek + Debug
{
    pub fn fourcc(&self) -> FourCC {
        self.fcc
    }
    pub fn size(&self) -> u64 {
        self.iobuff.size
    }
    pub fn read<'b>(&'b mut self) -> ChunkReader<'a, 'b, T> {
        self.iobuff.seek(io::SeekFrom::Start(0)).unwrap();
        ChunkReader { inner: self }
    }
}

#[derive(Debug)]
pub struct ChunkReader<'a, 'b, T>
    where T: 'a + Read + Seek + Debug,
          'a: 'b
{
    inner: &'b mut Chunk<'a, T>,
}

impl<'a, 'b, T> Read for ChunkReader<'a, 'b, T>
    where T: 'a + Read + Seek + Debug,
          'a: 'b
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.iobuff.read(buf)
    }
}
