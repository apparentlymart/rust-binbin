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
    let buf = Vec::<u8>::new();
    let cursor = std::io::Cursor::new(buf);
    let mut w = little_endian(cursor);
    let cstr = std::ffi::CStr::from_bytes_with_nul(b"howdy\0").unwrap();
    w.write(0xfeedfacedeadbeef as u64).unwrap();
    w.write(0xdeedbead as u32).unwrap();
    w.write(0x1234 as u16).unwrap();
    w.write(0xff as u8).unwrap();
    w.write(&b"hello"[..]).unwrap();
    w.write(cstr).unwrap();
    let buf = w.finalize().unwrap().into_inner();
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
