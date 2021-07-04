use std::io::{Read, Take};

/// The [`Read`](Read) implementation used with
/// [`Writer::derive`](crate::Writer::derive).
pub struct DeriveRead<'a, R>
where
    R: Read,
{
    r: Take<&'a mut R>,
}

impl<'a, R> DeriveRead<'a, R>
where
    R: Read,
{
    pub(crate) fn new(r: &'a mut R, limit: u64) -> Self {
        Self { r: r.take(limit) }
    }
}

impl<'a, R> Read for DeriveRead<'a, R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        self.r.read(buf)
    }
}
