use binrw::BinRead;

use crate::adabool;

use super::string::String;

#[derive(BinRead)]
pub enum TextHistoryType {
    #[br(magic = -1i8)]
    None {
        #[br(map = adabool)]
        has_culture_invariant_string: bool,
        #[br(if(has_culture_invariant_string))]
        text_data: String,
    },
    #[br(magic = 0i8)]
    Base {
        namespace: String,
        key: String,
        source_string: String,
    },
}

pub struct Text {
    pub flags: u32,
    pub history_type: TextHistoryType,
}
