use binrw::BinRead;

#[derive(BinRead)]
pub struct Transform {
    pub rotation: [f32; 4],
    pub translation: [f32; 3],
    pub scale: [f32; 3],
}
