pub trait Endian: private::Sealed {
    fn write_integer(v: u64, into: &mut [u8]);
}

pub enum LittleEndian {}

impl Endian for LittleEndian {
    fn write_integer(v: u64, into: &mut [u8]) {
        let l = into.len();
        for i in 0..l {
            into[i] = (v >> (8 * i)) as u8;
        }
    }
}

pub enum BigEndian {}

impl Endian for BigEndian {
    fn write_integer(v: u64, into: &mut [u8]) {
        let l = into.len();
        for i in 0..l {
            let shift = 8 * (l - i - 1);
            into[i] = (v >> shift) as u8;
        }
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::BigEndian {}
    impl Sealed for super::LittleEndian {}
}
