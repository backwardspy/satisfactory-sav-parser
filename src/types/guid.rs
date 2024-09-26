use binrw::BinRead;

#[derive(BinRead)]
pub struct Guid {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}
