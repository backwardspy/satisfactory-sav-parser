use binrw::BinRead;

use crate::adabool;

use super::{object_base_save_header::ObjectBaseSaveHeader, transform::Transform};

#[derive(BinRead)]
pub struct ActorSaveHeader {
    pub object_header: ObjectBaseSaveHeader,
    #[br(map = adabool)]
    pub need_transform: bool,
    pub transform: Transform,
    #[br(map = adabool)]
    pub was_placed_in_level: bool,
}
