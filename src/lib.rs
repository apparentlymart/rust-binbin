/// A utility library for more easily writing out structured data in
/// arbitrary binary file formats.
///
/// This library is particularly suited for formats that include offsets
/// to other parts of the file contents, or need to specify the size of
/// sub-sections before producing those subsections. It includes a mechaism
/// for labelling particular offsets and then including placeholders for
/// values derived from those offsets which can be updated later once their
/// results are known.
use std::io::{Result, Seek, Write};

pub mod endian;
pub mod pack;

#[cfg(test)]
mod tests;

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
pub struct Writer<'a, W, E>
where
    W: 'a + Seek + Write,
    E: Endian,
{
    w: W,
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
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn write<V: pack::IntoPack>(&mut self, v: V) -> Result<usize> {
        use pack::Pack;
        let v = v.into_pack();
        let l = v.len();
        let mut buf = vec![0 as u8; l];
        v.pack_into_slice::<E>(&mut buf[..]);
        self.w.write(&buf[..])
    }

    pub fn subregion<F>(&mut self, f: F) -> Result<std::ops::Range<u64>>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        let start_pos = self.w.stream_position()?;
        f(self)?;
        let end_pos = self.w.stream_position()?;
        Ok(start_pos..end_pos)
    }

    pub fn pending_label(&'a mut self) -> Label<'a> {
        todo!();
    }

    /// Consumes a previously-created label and sets its final value to
    /// the current offset, returning that offset.
    pub fn finalize_label(&'a mut self, label: Label<'a>) -> usize {
        todo!();
    }

    pub fn pending_value<T>(&'a mut self, init: T) -> Value<'a, T> {
        todo!();
    }

    pub fn update_value<T>(&'a mut self, value: Value<'a, T>, new: T) -> Value<'a, T> {
        todo!();
    }

    pub fn finalize_value<T>(&'a mut self, value: Value<'a, T>, fin: T) -> T {
        todo!();
    }

    fn finalize(self) -> Result<W> {
        // TODO: Also finish up all of the deferred values.
        Ok(self.w)
    }
}

impl<'a, T> Writer<'a, T, LittleEndian>
where
    T: Seek + Write,
{
    pub fn to_big_endian(self) -> Writer<'a, T, BigEndian> {
        Writer {
            w: self.w,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Writer<'a, T, BigEndian>
where
    T: Seek + Write,
{
    pub fn to_little_endian(self) -> Writer<'a, T, LittleEndian> {
        Writer {
            w: self.w,
            _phantom: std::marker::PhantomData,
        }
    }
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

pub struct Label<'a> {
    pe: PendingEntry<'a, usize>,
}

impl<'a> Pending<'a> for Label<'a> {
    type Value = usize;

    fn pending_entry(&self) -> PendingEntry<'a, usize> {
        self.pe
    }
}

pub struct Value<'a, T> {
    pe: PendingEntry<'a, T>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: 'a> Pending<'a> for Value<'a, T> {
    type Value = T;

    fn pending_entry(&self) -> PendingEntry<'a, Self::Value> {
        self.pe
    }
}

pub trait Pending<'a> {
    type Value: 'a;

    fn pending_entry(&self) -> PendingEntry<'a, Self::Value>;
}

pub struct PendingEntry<'a, T> {
    idx: usize,

    // We carry T here just to help Writer provide a type-safe interface,
    // even though our internal table isn't type-aware.
    phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Clone for PendingEntry<'a, T> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            phantom: self.phantom,
        }
    }
}

impl<'a, T> Copy for PendingEntry<'a, T> {}
