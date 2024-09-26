use binrw::BinRead;

use super::{object_base_save_header::ObjectBaseSaveHeader, string::String};

#[derive(BinRead)]
pub struct ObjectSaveHeader {
    pub base_header: ObjectBaseSaveHeader,
    pub outer_path_name: String,
}
