use super::*;

// Borrowed from https://crates.io/crates/assert_hex, under MIT license.
macro_rules! assert_eq_hex {
    ($left:expr, $right:expr $(,)?) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:#x?}`,
 right: `{:#x?}`"#, &*left_val, &*right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:#x?}`,
 right: `{:#x?}`: {}"#, &*left_val, &*right_val,
                           format_args!($($arg)+))
                }
            }
        }
    });
}

#[test]
fn immediate_only_write_little_endian() {
    let mut buf = Vec::<u8>::new();
    let got_ret = write_vec_le(&mut buf, |w| {
        let cstr = std::ffi::CStr::from_bytes_with_nul(b"howdy\0").unwrap();
        w.write(0xfeedfacedeadbeef as u64)?;
        w.write(0xdeedbead as u32)?;
        w.write(0x1234 as u16)?;
        w.write(0xff as u8)?;
        w.write(&b"hello"[..])?;
        w.write(cstr)?;
        Ok(9)
    })
    .unwrap();
    assert_eq!(got_ret, 9);
    assert_eq_hex!(
        buf,
        vec![
            0xef, 0xbe, 0xad, 0xde, 0xce, 0xfa, 0xed, 0xfe, // u64
            0xad, 0xbe, 0xed, 0xde, // u32
            0x34, 0x12, // u16
            0xff, // u8
            b'h', b'e', b'l', b'l', b'o', // &[u8]
            b'h', b'o', b'w', b'd', b'y', 0x00, // &std::ffi::CStr
        ]
    );
}

#[test]
fn immediate_only_write_big_endian() {
    let mut buf = Vec::<u8>::new();
    write_vec_be(&mut buf, |w| {
        let cstr = std::ffi::CStr::from_bytes_with_nul(b"howdy\0").unwrap();
        w.write(0xfeedfacedeadbeef as u64)?;
        w.write(0xdeedbead as u32)?;
        w.write(0x1234 as u16)?;
        w.write(0xff as u8)?;
        w.write(&b"hello"[..])?;
        w.write(cstr)?;
        Ok(())
    })
    .unwrap();
    assert_eq_hex!(
        buf,
        vec![
            0xfe, 0xed, 0xfa, 0xce, 0xde, 0xad, 0xbe, 0xef, // u64
            0xde, 0xed, 0xbe, 0xad, // u32
            0x12, 0x34, // u16
            0xff, // u8
            b'h', b'e', b'l', b'l', b'o', // &[u8]
            b'h', b'o', b'w', b'd', b'y', 0x00, // &std::ffi::CStr
        ]
    );
}

#[test]
fn deferred_write_big_endian() {
    let mut buf = Vec::<u8>::new();
    write_vec_be(&mut buf, |w| {
        let defer = w.deferred(0xffffffff as u32);
        w.write(0xfefefefe as u32)?;
        w.write_placeholder(defer)?;
        w.write(0xfefefefe as u32)?;
        w.write_placeholder(defer)?;
        w.write(0xfefefefe as u32)?;
        let fin = w.resolve(defer, 0x12345678)?;
        w.write(fin)?;
        Ok(())
    })
    .unwrap();
    assert_eq_hex!(
        buf,
        vec![
            0xfe, 0xfe, 0xfe, 0xfe, // immediate
            0x12, 0x34, 0x56, 0x78, // deferred
            0xfe, 0xfe, 0xfe, 0xfe, // immediate
            0x12, 0x34, 0x56, 0x78, // deferred
            0xfe, 0xfe, 0xfe, 0xfe, // immediate
            0x12, 0x34, 0x56, 0x78, // final
        ]
    );
}

#[test]
fn align() {
    let mut buf = Vec::<u8>::new();
    write_vec_be(&mut buf, |w| {
        w.write(0x45 as u8)?;
        w.set_padding(0x3d);
        w.align(4)?;
        w.write(0xfefefefe as u32)?;
        w.align(4)?; // already aligned, so does nothing
        w.write(0x67676767 as u32)?;
        Ok(())
    })
    .unwrap();
    assert_eq_hex!(
        buf,
        vec![
            0x45, // initial byte
            0x3d, 0x3d, 0x3d, // alignment padding
            0xfe, 0xfe, 0xfe, 0xfe, // first u32
            0x67, 0x67, 0x67, 0x67, // second u32
        ]
    );
}

#[test]
fn derive() {
    let mut buf = Vec::<u8>::new();
    let ret_sum = write_vec_be(&mut buf, |w| {
        w.write(0x01 as u8)?;
        w.write(0x02 as u8)?;
        let sum = w.derive(0..2, |r| {
            use std::io::Read;
            let mut buf: [u8; 2] = [0, 0];
            r.read(&mut buf)?;

            let mut buf2: [u8; 1] = [0];
            if let Ok(n) = r.read(&mut buf2) {
                if n != 0 {
                    panic!("second read should not succeed");
                }
            }

            Ok(buf[0] + buf[1])
        })?;
        w.write(sum)?;
        Ok(sum)
    })
    .unwrap();
    assert_eq_hex!(buf, vec![0x01, 0x02, 0x03]);
    assert_eq!(ret_sum, 3);
}
