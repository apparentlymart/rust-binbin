use crate::{BigEndian, Endian, LittleEndian};

/// Trait implemented by types that can be packed into a sequence of bytes
/// to be written into a file.
pub trait Pack {
    /// Returns the number of bytes the value would be packed into.
    fn len(&self) -> usize;

    /// Packs the value into the given slice, whose length must match the
    /// return value of [`Pack::len`](Self::len).
    ///
    /// If given a slice of the wrong length, the behavior is undefined,
    /// including possible panics.
    fn pack_into_slice<E: Endian>(&self, into: &mut [u8]);
}

/// Marks a particular value has being forced as little-endian when encoded,
/// and thus ignoring whichever endianness is selected as the default for a
/// [`Writer`](super::Writer).
///
/// Passing an [`EndianOverride`](EndianOverride) to this function will have
/// no effect on its existing forced endianness.
pub fn to_little_endian<P: Pack + Sized>(v: P) -> EndianOverride<P, LittleEndian> {
    EndianOverride {
        v: v,
        phantom: std::marker::PhantomData,
    }
}

/// Marks a particular value has being forced as big-endian when encoded,
/// and thus ignoring whichever endianness is selected as the default for a
/// [`Writer`](super::Writer).
///
/// Passing an [`EndianOverride`](EndianOverride) to this function will have
/// no effect on its existing forced endianness.
pub fn to_big_endian<P: Pack + Sized>(v: P) -> EndianOverride<P, BigEndian> {
    EndianOverride {
        v: v,
        phantom: std::marker::PhantomData,
    }
}

impl Pack for u8 {
    fn len(&self) -> usize {
        1
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        // Endian doesn't matter for only one byte!
        buf[0] = *self
    }
}

impl Pack for i8 {
    fn len(&self) -> usize {
        (*self as u8).len()
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        (*self as u8).pack_into_slice::<E>(buf)
    }
}

impl Pack for u16 {
    fn len(&self) -> usize {
        2
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        E::write_integer(*self as u64, &mut buf[0..2])
    }
}

impl Pack for i16 {
    fn len(&self) -> usize {
        (*self as u16).len()
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        (*self as u16).pack_into_slice::<E>(buf)
    }
}

impl Pack for u32 {
    fn len(&self) -> usize {
        4
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        E::write_integer(*self as u64, &mut buf[0..4])
    }
}

impl Pack for i32 {
    fn len(&self) -> usize {
        (*self as u32).len()
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        (*self as u32).pack_into_slice::<E>(buf)
    }
}

impl Pack for u64 {
    fn len(&self) -> usize {
        8
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        E::write_integer(*self as u64, &mut buf[0..8])
    }
}

impl Pack for i64 {
    fn len(&self) -> usize {
        (*self as u64).len()
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        (*self as u64).pack_into_slice::<E>(buf)
    }
}

/// [`CStr`](std::ffi::CStr) values pack as null-terminated strings, with no
/// additional padding other than the null terminator.
impl Pack for std::ffi::CStr {
    fn len(&self) -> usize {
        self.to_bytes_with_nul().len()
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self.to_bytes_with_nul());
    }
}

/// [`CStr`](std::ffi::CStr) values pack as null-terminated strings, with no
/// additional padding other than the null terminator.
impl Pack for &std::ffi::CStr {
    fn len(&self) -> usize {
        self.to_bytes_with_nul().len()
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self.to_bytes_with_nul());
    }
}

// `[u8]` values pack by just copying the bytes verbatim into the output
// buffer.
impl Pack for [u8] {
    fn len(&self) -> usize {
        self.len()
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self);
    }
}

// `&[u8]` values pack by just copying the bytes verbatim into the output
// buffer.
impl Pack for &[u8] {
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    fn pack_into_slice<E: Endian>(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self);
    }
}

/// A special [`Pack`](Pack) implementation that forces a particular
/// endianness for some other wrapped value, regardless of the endianness
/// selected for the writer these values are passed to.
///
/// `EndianOverride` is the return type for both
/// [`to_little_endian`](to_little_endian) and
/// [`to_big_endian`](to_big_endian).
pub struct EndianOverride<T: Pack, E: Endian> {
    v: T,
    phantom: std::marker::PhantomData<E>,
}

impl<T: Pack, E: Endian> Pack for EndianOverride<T, E> {
    fn len(&self) -> usize {
        self.v.len()
    }

    /// Despite the usual meaning of `pack_into_slice`, this implementation
    /// ignores the method-level endianness and honors the type-level
    /// endianness instead.
    fn pack_into_slice<Ignored: Endian>(&self, buf: &mut [u8]) {
        self.v.pack_into_slice::<E>(buf)
    }
}

/// A trait implemented by types that can convert to types that implement
/// [`Pack`](Pack).
pub trait IntoPack {
    /// The type that `into_pack` returns.
    type PackType: Pack;

    /// Converts the given value into a suitable [`Pack`](Pack)-implementing
    /// type.
    fn into_pack(self) -> Self::PackType;
}

/// All [`Pack`](Pack) implementers also implement [`IntoPack`](IntoPack),
/// just returning themselves verbatim.
impl<P: Pack> IntoPack for P {
    type PackType = P;

    fn into_pack(self) -> Self::PackType {
        self
    }
}
