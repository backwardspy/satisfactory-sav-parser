use binrw::BinRead;

use super::{map::Map, string::Name};

#[derive(BinRead)]
pub struct WPGridValidationData {
    pub cell_size: i32,
    pub grid_hash: u32,
    pub cell_hashes: Map<Name, u32>,
}
