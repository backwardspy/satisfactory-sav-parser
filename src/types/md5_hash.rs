use binrw::BinRead;

use crate::adabool;

#[derive(BinRead)]
pub struct MD5Hash {
    #[br(map = adabool)]
    pub is_valid: bool,
    #[br(if(is_valid))]
    pub bytes: [u8; 16],
}
