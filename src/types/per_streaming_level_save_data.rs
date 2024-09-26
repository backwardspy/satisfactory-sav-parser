use binrw::BinRead;

use super::{array::Array, object_reference_disc::ObjectReferenceDisc};

#[derive(BinRead)]
pub struct PerStreamingLevelSaveData {
    toc_blob: Array<u8, i64>,
    data_blob: Array<u8, i64>,
    destroyed_actors: Array<ObjectReferenceDisc>,
}
