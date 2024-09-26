use binrw::BinRead;

use super::{array::Array, object_reference_disc::ObjectReferenceDisc};

#[derive(BinRead)]
pub struct UnresolvedWorldSaveData {
    pub destroyed_actors: Array<ObjectReferenceDisc>,
}
