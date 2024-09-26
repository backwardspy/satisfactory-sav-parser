use std::{
    io::{Read, Seek},
    marker::PhantomData,
};

use binrw::{BinRead, BinResult, Endian, NamedArgs};

#[derive(Clone, Default, NamedArgs)]
pub struct ArrayArgs<Inner: Clone> {
    /// The [arguments](binrw::BinRead::Args) for the inner type.
    pub inner: Inner,
}

pub trait ArraySizeType {
    fn into_usize(self) -> usize;
}

#[derive(Debug)]
pub struct Array<T, SizeType = i32>(pub Vec<T>, PhantomData<SizeType>);

impl<T, SizeType> BinRead for Array<T, SizeType>
where
    T: BinRead + 'static,
    for<'a> T::Args<'a>: Clone,
    SizeType: BinRead<Args<'static> = ()> + ArraySizeType,
{
    type Args<'a> = ArrayArgs<T::Args<'a>>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let count = SizeType::read_options(reader, endian, ())?;
        binrw::helpers::count_with(count.into_usize(), T::read_options)(reader, endian, args.inner)
            .map(|vec| Array(vec, PhantomData))
    }
}

impl ArraySizeType for i32 {
    fn into_usize(self) -> usize {
        self as usize
    }
}

impl ArraySizeType for i64 {
    fn into_usize(self) -> usize {
        self as usize
    }
}
