use std::{
    fmt,
    io::{Read, Seek},
};

use binrw::{BinRead, BinResult, Endian, NullString, NullWideString};

/// A variable-length byte sequence of UTF-encoded characters, null-terminated:
/// 4 byte signed integer length, encoded characters, null terminator.
/// If length is positive, the string is `length` UTF-8 bytes with one null byte at the end.
/// If length is negative, the string is `length * -2` UTF-16 bytes with two null bytes at the end.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub enum String {
    #[default]
    Empty,
    UTF8(Vec<u8>),
    UTF16(Vec<u16>),
}

pub type Name = String;

impl BinRead for String {
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
                assert_eq!(
                    string.0.len() + 2,
                    length as usize,
                    "expected utf-16 string of length {} but got {}",
                    length,
                    string.0.len()
                );
                String::UTF16(string.0)
            }
            std::cmp::Ordering::Equal => String::Empty,
            std::cmp::Ordering::Greater => {
                let string = <NullString>::read_options(reader, endian, ())?;
                assert_eq!(
                    string.0.len() + 1,
                    length as usize,
                    "expected utf-8 string of length {} but got {}",
                    length,
                    string.0.len()
                );
                String::UTF8(string.0)
            }
        })
    }
}

fn utf8_string(bytes: &[u8]) -> std::string::String {
    std::string::String::from_utf8_lossy(bytes).to_string()
}

fn utf16_string(bytes: &[u16]) -> std::string::String {
    std::string::String::from_utf16_lossy(bytes).to_string()
}

impl PartialEq<&str> for String {
    fn eq(&self, other: &&str) -> bool {
        match self {
            String::Empty => other.is_empty(),
            String::UTF8(v) => utf8_string(v) == *other,
            String::UTF16(v) => utf16_string(v) == *other,
        }
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            String::Empty => Ok(()),
            String::UTF8(v) => write!(f, "{}", utf8_string(v)),
            String::UTF16(v) => write!(f, "{}", utf16_string(v)),
        }
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "ADAString::Empty"),
            Self::UTF8(v) => write!(f, "ADAString::UTF8({:?})", utf8_string(v)),
            Self::UTF16(v) => write!(f, "ADAString::UTF16({:?})", utf16_string(v)),
        }
    }
}
