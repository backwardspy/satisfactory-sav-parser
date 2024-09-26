use binrw::BinRead;

use super::string::String;

#[derive(BinRead)]
pub struct ObjectReferenceDisc {
    pub level_name: String,
    pub path_name: String,
}
