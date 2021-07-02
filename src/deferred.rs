/// A placeholder for a value that we'll learn only later in our process of
/// writing out data.
#[derive(Copy, Clone)]
pub struct Deferred<'a, T> {
    pub(crate) idx: usize,
    pub(crate) initial: T,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Deferred<'a, T> {
    pub(crate) fn new(idx: usize, initial: T) -> Self {
        Self {
            idx: idx,
            initial: initial,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Deferred<'a, T>
where
    T: crate::pack::IntoPack,
    <T as crate::pack::IntoPack>::PackType: crate::pack::FixedLenPack,
{
    /// The size, in bytes, that the deferred value will take up when written.
    pub const PACK_LEN: usize =
        <<T as crate::pack::IntoPack>::PackType as crate::pack::FixedLenPack>::PACK_LEN;

    /// Returns the size, in bytes, that the deferred value will take up when
    /// written.
    pub fn pack_len(&mut self) -> usize {
        Self::PACK_LEN
    }
}
