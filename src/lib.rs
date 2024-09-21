//! https://satisfactory.fandom.com/wiki/Save_files
//!
//! Primitive data types:
//!
//! Byte
//! A single 8-bit byte that represents a signed integer between -128 and 127.
//!
//! Int
//! Four consecutive bytes in little-endian order that represent a signed integer between -2,147,483,648 and 2,147,483,647.
//!
//! Long
//! Eight consecutive bytes in little-endian order that represent a signed integer between -9,223,372,036,854,775,808 and 9,223,372,036,854,775,807.
//!
//! Float
//! Four consecutive bytes in little-endian order that represent a signed floating-point number with single precision, according to the binary32 format of IEEE 754.
//!
use std::io::{Read, Seek};

use binrw::{BinRead, BinReaderExt};
use thiserror::Error;
use types::ADAString;

mod types;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read save file")]
    BinRead(#[from] binrw::Error),
}

#[derive(Debug, BinRead)]
pub struct Header {
    pub version: i32,
    pub save_version: i32,
    pub build_version: i32,
    pub map_name: ADAString,
    pub map_options: ADAString,
    pub session_name: ADAString,
    pub seconds_played: i32,
    pub save_timestamp: i64,
    pub session_visibility: i8,
    pub editor_object_version: i32,
    pub mod_metadata: ADAString,
    pub mod_flags: i32,
    pub save_identifier: ADAString,
    pub is_partitioned_world: i32,
    pub md5_hash: [u8; 20],
    pub is_creative_mode_enabled: i32,
}

pub struct Parser<R> {
    data: R,
}

impl<R> Parser<R>
where
    R: Read + Seek,
{
    pub fn new(data: R) -> Self {
        Parser { data }
    }

    pub fn read_header(&mut self) -> Result<Header, Error> {
        self.data.read_le().map_err(Error::BinRead)
    }
}
