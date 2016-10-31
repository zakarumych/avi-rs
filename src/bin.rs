#![feature(question_mark)]
#![allow(unused_imports)]
extern crate avirs;

use std::fs::File;
use std::io::{self, Read, Seek};
use std::fmt::{self, Formatter, Display, Debug};

use avirs::riff::*;
use avirs::fourcc::FourCC;

struct MemSize(u64);
impl Display for MemSize {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let suffs = ["B", "KB", "MB", "GB", "TB", "PB"];
        let mut count = self.0;
        for suff in suffs.iter() {
            if count <= 9999 {
                return write!(f, "{}{}", count, suff);
            }
            count /= 1024;
        }
        write!(f, "{}EB", count)
    }
}

impl Debug for MemSize {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}B", self.0)
    }
}


const SPACES: &'static str = "                                                                ";

fn print_hier<T: io::Read + io::Seek>(mut list: List<T>, depth: usize) -> io::Result<()> {
    println!("{}LIST[{:?}]:{}", &SPACES[..depth * 2], list.fourcc(), MemSize(list.size()));
    for sub in list.iter() {
        match sub? {
            Node::Chunk(chunk) => {
                println!("{}CHUNK[{:?}]:{}",
                         &SPACES[..depth * 2 + 2],
                         chunk.fourcc(),
                         MemSize(chunk.size()));
            }
            Node::List(list) => {
                print_hier(list, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn test1(path: &str) -> io::Result<()> {
    let filename = path;
    let file = File::open(filename)?;

    let mut riff = Riff::new(file)?;
    for list in riff.iter() {
        print_hier(list?, 0)?;
    }
    let mut file = riff.release();
    let size = file.seek(io::SeekFrom::Current(0))?;
    println!("RIFF size: {}", size);
    Ok(())
}

fn test2(path: &str) -> io::Result<()> {
    use avirs::demuxer::*;
    let filename = path;
    let file = File::open(filename)?;

    let mut riff = Riff::new(file)?;

    let demuxer = Demuxer::from_riff(&mut riff);
    println!("{:?}", demuxer);
    Ok(())
}

fn main() {
    for arg in std::env::args() {
        println!("{}", arg);
    } 
    let arg = std::env::args().nth(1).unwrap();
    println!("{}", arg);
    println!("RIFF = {:?}", RIFF);
    println!("LIST = {:?}", LIST);
    test1(&arg).unwrap();
    //test2().unwrap();
}
