use crate::Endian;

pub trait Pack {
    const LENGTH: usize;

    fn pack_into_slice<E: Endian>(&self, into: &mut [u8]);
}

impl Pack for u8 {
    const LENGTH: usize = 1;

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        buf[0] = *self
    }
}

pub trait IntoPack {
    type PackType: Pack;

    fn into_pack(self) -> Self::PackType;
}

impl<P: Pack> IntoPack for P {
    type PackType = P;

    fn into_pack(self) -> Self::PackType {
        self
    }
}
