use binrw::BinRead;
use indexmap::IndexMap;
use std::hash::Hash;

pub struct Map<K, V>(pub IndexMap<K, V>);

impl<K, V> BinRead for Map<K, V>
where
    K: Hash + Eq + BinRead<Args<'static> = ()>,
    V: BinRead,
    for<'a> <V as BinRead>::Args<'a>: Clone,
{
    type Args<'a> = <V as BinRead>::Args<'a>;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let size = i32::read_options(reader, endian, ())?;
        let mut data = IndexMap::new();
        for _ in 0..size {
            let key = K::read_options(reader, endian, ())?;
            let value = V::read_options(reader, endian, args.clone())?;
            data.insert(key, value);
        }
        Ok(Map(data))
    }
}
