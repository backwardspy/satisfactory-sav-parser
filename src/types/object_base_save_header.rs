use binrw::BinRead;

use super::{object_reference_disc::ObjectReferenceDisc, string::String};

#[derive(BinRead)]
pub struct ObjectBaseSaveHeader {
    pub class_name: String,
    pub reference: ObjectReferenceDisc,
}
