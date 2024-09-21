use std::{
    fmt,
    io::{Read, Seek},
};

use binrw::{BinRead, BinResult, Endian, NullString, NullWideString};

/// A variable-length byte sequence of UTF-encoded characters, null-terminated:
/// 4 byte signed integer length, encoded characters, null terminator.
/// If length is positive, the string is `length` UTF-8 bytes with one null byte at the end.
/// If length is negative, the string is `length * -2` UTF-16 bytes with two null bytes at the end.
pub enum ADAString {
    Empty,
    UTF8(Vec<u8>),
    UTF16(Vec<u16>),
}

impl BinRead for ADAString {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let length = <i32>::read_options(reader, endian, ())?;
        Ok(match length.cmp(&0) {
            std::cmp::Ordering::Less => {
                let length = length * -2;
                let string = <NullWideString>::read_options(reader, endian, ())?;
                assert_eq!(string.0.len() + 2, length as usize);
                ADAString::UTF16(string.0)
            }
            std::cmp::Ordering::Equal => ADAString::Empty,
            std::cmp::Ordering::Greater => {
                let string = <NullString>::read_options(reader, endian, ())?;
                assert_eq!(string.0.len() + 1, length as usize);
                ADAString::UTF8(string.0)
            }
        })
    }
}

impl fmt::Display for ADAString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ADAString::Empty => Ok(()),
            ADAString::UTF8(string) => write!(f, "{}", String::from_utf8_lossy(string)),
            ADAString::UTF16(string) => write!(f, "{}", String::from_utf16_lossy(string)),
        }
    }
}

impl fmt::Debug for ADAString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "ADAString::Empty"),
            Self::UTF8(v) => write!(f, "ADAString::UTF8({:?})", String::from_utf8_lossy(v)),
            Self::UTF16(v) => write!(f, "ADAString::UTF16({:?})", String::from_utf16_lossy(v)),
        }
    }
}
