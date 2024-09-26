use binrw::BinRead;

use super::{array::Array, map::Map, object_reference_disc::ObjectReferenceDisc, string::String};

#[derive(BinRead)]
pub struct PersistentAndRuntimeSaveData {
    pub toc_blob: Array<u8, i64>,
    pub data_blob: Array<u8, i64>,
    pub level_to_destroyed_actors: Map<String, Array<ObjectReferenceDisc>>,
}
