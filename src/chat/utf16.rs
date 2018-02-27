use std::io::prelude::*;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::io::Result;
use std::fs::File;
use std::io::Cursor;

extern crate encoding;
use self::encoding::{Encoding, DecoderTrap};
use self::encoding::all::UTF_16LE as UTF16;

type UTF16Buffer = BufReader<Cursor<Vec<u8>>>;

pub struct UTF16File;


impl UTF16File {
    pub fn open(path: String) -> Result<UTF16Buffer> {
        let content = try!(UTF16File::read_utf18(&path));
        return Ok(BufReader::new(Cursor::new(content)));
    }

    fn read_utf18(path: &str) -> Result<Vec<u8>> {
        let mut file = try!(File::open(path));
        let mut buf: Vec<u8> = vec![]; 
        try!(file.read_to_end(&mut buf));
        drop(file);
        match UTF16.decode(&buf, DecoderTrap::Ignore) {
            Ok(content) => { Ok(content.into_bytes()) } ,
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        }
    }
}
