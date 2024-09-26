use binrw::BinRead;

use super::{map::Map, string::Name, wp_grid_validation_data::WPGridValidationData};

#[derive(BinRead)]
pub struct WorldPartitionValidationData {
    pub grids: Map<Name, WPGridValidationData>,
}
