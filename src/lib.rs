/// A utility library for more easily writing out structured data in
/// arbitrary binary file formats.
///
/// This library is particularly suited for formats that include offsets
/// to other parts of the file contents, or need to specify the size of
/// sub-sections before producing those subsections. It includes a mechaism
/// for labelling particular offsets and then including placeholders for
/// values derived from those offsets which can be updated later once their
/// results are known.
use std::io::{Read, Result, Seek, Write};

/// Representation of values to be determined later.
pub mod deferred;

/// Types for representing endianness.
pub mod endian;

/// Traits for serializing data for [`Writer::write`](Writer::write).
pub mod pack;

/// Types used with [`Writer::derive`](Writer::derive).
pub mod derive;

#[cfg(test)]
mod tests;

use deferred::Deferred;
use endian::{BigEndian, Endian, LittleEndian};

/// Writes arbitrary binary data to the given writer `w` using the given
/// function `f`, where writes will be little-endian by default.
///
/// The given function -- generally a closure -- establishes the lifetime for
/// any deferred values, so that `write_le` can ensure that all
/// deferred values are taken care of before returning.
pub fn write_le<W, F>(w: &mut W, f: F) -> Result<usize>
where
    W: Write + Seek,
    F: FnOnce(&mut Writer<&mut W, LittleEndian>) -> Result<()>,
{
    write::<_, _, LittleEndian>(w, f)
}

/// Writes arbitrary binary data into a byte vector using the given
/// function `f`, writing little-endian by default.
pub fn write_vec_le<F>(into: &mut Vec<u8>, f: F) -> Result<()>
where
    F: FnOnce(&mut Writer<&mut std::io::Cursor<&mut Vec<u8>>, LittleEndian>) -> Result<()>,
{
    write_buf::<_, LittleEndian>(into, f)
}

/// Writes arbitrary binary data to the given writer `w` using the given
/// function `f`, where writes will be big-endian by default.
///
/// The given function -- generally a closure -- establishes the lifetime for
/// any deferred values, so that `write_be` can ensure that all
/// deferred values are taken care of before returning.
pub fn write_be<W, F>(w: &mut W, f: F) -> Result<usize>
where
    W: Write + Seek,
    F: FnOnce(&mut Writer<&mut W, BigEndian>) -> Result<()>,
{
    write::<_, _, BigEndian>(w, f)
}

/// Writes arbitrary binary data into a byte vector using the given
/// function `f`, writing big-endian by default.
pub fn write_vec_be<F>(into: &mut Vec<u8>, f: F) -> Result<()>
where
    F: FnOnce(&mut Writer<&mut std::io::Cursor<&mut Vec<u8>>, BigEndian>) -> Result<()>,
{
    write_buf::<_, BigEndian>(into, f)
}

fn write<W, F, E>(w: &mut W, f: F) -> Result<usize>
where
    W: Write + Seek,
    F: FnOnce(&mut Writer<&mut W, E>) -> Result<()>,
    E: Endian,
{
    let start_pos = w.stream_position()?;
    let mut wr = Writer::new(w);
    f(&mut wr)?;
    let w = wr.finalize()?;
    let end_pos = w.stream_position()?;
    return Ok((end_pos - start_pos) as usize);
}

fn write_buf<F, E>(into: &mut Vec<u8>, f: F) -> Result<()>
where
    F: FnOnce(&mut Writer<&mut std::io::Cursor<&mut Vec<u8>>, E>) -> Result<()>,
    E: Endian,
{
    let mut cursor = std::io::Cursor::new(into);
    write(&mut cursor, f)?;
    return Ok(());
}

/// Wraps a seekable writer with extra functions to conveniently write
/// data in various common binary formats and keep track of labelled offsets
/// to help calculate section sizes and object positions.
///
/// Each writer has an endianness as part of its type, which dictates how it
/// will write out multi-byte values. The endianness is built into the writer
/// because most formats exclusively use a single endianness throughout, but
/// for situations where that isn't true you can use an endian override
/// for a particular value and thus ignore the writer's default.
///
/// During writing the underlying writer will contain placeholder data for
/// any deferred values, which will then be overwritten with true values during
/// finalization. If the underlying writer is a file on disk then other
/// applications may be able to observe the placeholder values if they happen
/// to inspect the file while it's under construction.
///
/// If any operation on a `Writer` returns an error, the underlying stream is
/// left in an undefined state and the user should cease further use of the
/// writer and treat the result as invalid.
pub struct Writer<'a, W, E>
where
    W: 'a + Seek + Write,
    E: Endian,
{
    w: W,
    map: Vec<Vec<u64>>,
    pad: u8,
    _phantom: std::marker::PhantomData<&'a E>,
}

impl<'a, W, E> Writer<'a, W, E>
where
    W: Seek + Write,
    E: Endian,
{
    fn new(w: W) -> Self {
        Self {
            w: w,
            map: Vec::new(),
            pad: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Writes a value to the current position in the output.
    ///
    /// `write` can accept any value that implements
    /// [`IntoPack`](pack::IntoPack), and will write the result from packing
    /// the value to the underlying stream.
    pub fn write<V: pack::IntoPack>(&mut self, v: V) -> Result<usize> {
        write_intopack_value::<_, _, E>(&mut self.w, v)
    }

    /// Writes a placeholder for the given deferred slot to the current
    /// position in the output.
    ///
    /// At some later point you should pass the same deferred slot to
    /// [`resolve`](Self::resolve) along with its final value, at which point
    /// the placeholder will be overwritten.
    pub fn write_placeholder<T>(&mut self, deferred: Deferred<'a, T>) -> Result<usize>
    where
        T: pack::IntoPack,
        <T as pack::IntoPack>::PackType: pack::FixedLenPack,
    {
        // We write the slot's initial value for now, but also track
        // in self.map where this was so that resolving it later can
        // overwrite with the final value.
        let pos = self.position()?;
        let size = write_intopack_value::<_, _, E>(&mut self.w, deferred.initial)?;
        self.map[deferred.idx].push(pos);
        return Ok(size);
    }

    /// A shorthand combining [`deferred`](Self::deferred) and
    /// [`write_placeholder`](Self::write_placeholder), to create a new
    /// deferred slot and write a placeholder for it in a single call.
    pub fn write_deferred<T>(&mut self, initial: T) -> Result<Deferred<'a, T>>
    where
        T: pack::IntoPack + Copy,
        <T as pack::IntoPack>::PackType: pack::FixedLenPack,
    {
        let ret = self.deferred(initial);
        self.write_placeholder(ret)?;
        Ok(ret)
    }

    /// Moves the current stream position forward to a position aligned to the
    /// given number of bytes, writing padding bytes as necessary. Returns the
    /// number of padding bytes written.
    ///
    /// A new `Writer` defaults to using zeros for padding. Use
    /// [`set_padding`](Self::set_padding) to override the padding byte for
    /// future writes, if needed.
    pub fn align(&mut self, n: usize) -> Result<usize> {
        let pos = self.position()?;
        let ofs = pos % (n as u64);
        if ofs == 0 {
            return Ok(0);
        }
        let inc = ((n as u64) - ofs) as usize;
        for _ in 0..inc {
            self.w.write(std::slice::from_ref(&self.pad))?;
        }
        Ok(inc)
    }

    /// Returns the current write position in the underlying writer.
    ///
    /// Use this with [`resolve`](Self::resolve) to resolve a deferred slot that
    /// ought to contain the offset of whatever new content you are about to
    /// write.
    pub fn position(&mut self) -> Result<u64> {
        self.w.stream_position()
    }

    /// Creates a region of the output whose final bounds must be known for
    /// use elsewhere in the output.
    ///
    /// If the given function completes successfully, `subregion` returns
    /// a range describing the start and end positions of the subregion
    /// in the underlying stream.
    pub fn subregion<F>(&mut self, f: F) -> Result<std::ops::Range<u64>>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        let start_pos = self.w.stream_position()?;
        f(self)?;
        let end_pos = self.w.stream_position()?;
        Ok(start_pos..end_pos)
    }

    /// Creates a slot for a value whose resolution will come later in
    /// the process of writing all of the data.
    ///
    /// You can use [`write_placeholder`](Self::write_placeholder) to reserve
    /// an area of the output where the final value will eventually be
    /// written. The reserved space will initially contain the value
    /// given in `initial`.
    ///
    /// Call [`resolve`](Self::resolve) to set the final value for this slot.
    /// That will then overwrite any placeholders written earlier with
    /// the final value.
    pub fn deferred<T>(&mut self, initial: T) -> Deferred<'a, T>
    where
        T: pack::IntoPack + Copy,
        <T as pack::IntoPack>::PackType: pack::FixedLenPack,
    {
        let next_idx = self.map.len();
        self.map.push(Vec::new());
        return deferred::Deferred::new(next_idx, initial);
    }

    /// Assigns a final value to a deferred data slot previously established
    /// using [`deferred`](deferred).
    pub fn resolve<T>(&mut self, deferred: Deferred<'a, T>, v: T) -> Result<T>
    where
        T: pack::IntoPack + Copy,
        <T as pack::IntoPack>::PackType: pack::FixedLenPack,
    {
        let reset_pos = self.position()?; // will restore at the end
        let result = self.write_resolved_values(deferred, v);
        self.w.seek(std::io::SeekFrom::Start(reset_pos))?;
        result
    }

    /// Changes the padding value used for future calls to
    /// [`align`](Self::align), and possibly for other functionality added
    /// in future that might also create padding.
    pub fn set_padding(&mut self, v: u8) {
        self.pad = v;
    }

    fn write_resolved_values<T>(&mut self, deferred: Deferred<'a, T>, v: T) -> Result<T>
    where
        T: pack::IntoPack + Copy,
        <T as pack::IntoPack>::PackType: pack::FixedLenPack,
    {
        let pv = v.into_pack();
        for offset in &self.map[deferred.idx] {
            self.w.seek(std::io::SeekFrom::Start(*offset))?;
            write_pack_value::<_, _, E>(&mut self.w, &pv)?;
        }
        Ok(v)
    }

    fn finalize(mut self) -> Result<W> {
        self.w.flush()?;
        Ok(self.w)
    }
}

impl<'a, W, E> Writer<'a, W, E>
where
    W: Seek + Write + Read,
    E: Endian,
{
    /// Derive a value from an already-written region of the underlying
    /// stream.
    ///
    /// This function is available only for streams that implement
    /// [`Read`](Read) in addition to the usually-required [`Write`](Write)
    /// and [`Seek`](Seek).
    ///
    /// The given function recieves a reader over the requested region, and
    /// can return any value derived from the contents of that region.
    /// For example, some binary formats include checksums to help with
    /// error detection, and you could potentially use `derive` over the
    /// relevant subregion to calculate such a checksum.
    pub fn derive<F, T>(&mut self, rng: std::ops::Range<u64>, f: F) -> Result<T>
    where
        F: FnOnce(&mut derive::DeriveRead<W>) -> Result<T>,
    {
        if rng.end < rng.start {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        }
        let len = rng.end - rng.start;
        let after_pos = self.position()?;
        self.w.seek(std::io::SeekFrom::Start(rng.start))?;
        let w = &mut self.w;
        let mut lr = derive::DeriveRead::new(w, len);
        let ret = f(&mut lr);
        self.w.seek(std::io::SeekFrom::Start(after_pos))?;
        return ret;
    }
}

fn write_intopack_value<W: Write, V: pack::IntoPack, E: Endian>(mut w: W, v: V) -> Result<usize> {
    let v = v.into_pack();
    write_pack_value::<_, _, E>(&mut w, &v)
}

fn write_pack_value<W: Write, V: pack::Pack, E: Endian>(w: &mut W, v: &V) -> Result<usize> {
    let l = v.pack_len();
    let mut buf = vec![0 as u8; l];
    v.pack_into_slice::<E>(&mut buf[..]);
    w.write(&buf[..])
}

impl<'a, T, E> Write for Writer<'a, T, E>
where
    T: Seek + Write,
    E: Endian,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.w.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.w.flush()
    }
}
