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

/// Wraps a seekable writer with extra functions to conveniently write
/// data in various common binary formats and keep track of labelled offsets
/// to help calculate section sizes and object positions.
///
/// Each writer has an endianness as part of its type, which dictates how it
/// will write out multi-byte values. The endianness is built into the writer
/// because most formats exclusively use a single endianness throughout, but
/// for situations where that isn't true you can use
/// `to_big_endian` or `to_little_endian` to switch an existing writer to
/// a different endianness without disturbing anything previously written.
///
/// The expected usage pattern for a `Writer` is to create one wrapping a
/// general seekable writer using either [`little_endian`](little_endian)
/// or [`big_endian`](big_endian), then write zero or more values into it,
/// and then to finalize it using method `finalize` to resolve any
/// forward-declarations and recover the original wrapped writer.
///
/// During writing the underlying writer will contain placeholder values for
/// any forward-declared values, which will then be overwritten with true
/// values during finalization. If the underlying writer is a file on disk
/// then other applications may be able to observe the placeholder values if
/// they happen to inspect the file while it's under construction.
pub struct Writer<'a, W, E>
where
    W: 'a + Seek + Write,
    E: Endian,
{
    w: W,
    _phantom: std::marker::PhantomData<&'a E>,
}

/// Wraps the given seekable writer into a little-endian binbin Writer.
pub fn little_endian<'a, W: 'a + Seek + Write>(w: W) -> Writer<'a, W, LittleEndian> {
    Writer::new(w)
}

/// Wraps the given seekable writer into a big-endian binbin Writer.
pub fn big_endian<'a, W: 'a + Seek + Write>(w: W) -> Writer<'a, W, BigEndian> {
    Writer::new(w)
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

    pub fn finalize(self) -> Result<W> {
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
