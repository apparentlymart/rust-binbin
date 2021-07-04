# binbin for Rust

`binbin` is a library for writing structured binary data in arbitrary formats.

It has the following useful features, implemented in terms of
`std::io::Write`, `std::io::Seek`, and `std::io::Read`:

- Automatically encoding Rust integer types as either little-endian or
  big-endian.
- Insert padding to align to a particular number of bytes, such as padding
  to the nearest four-byte increment.
- Insert placeholders for values that won't be known until later on in the
  writing process, such as the size of some variable-sized data to follow,
  and then update them in-place once you know the final value.
- Derive new values from blocks of data already written, such as including
  a checksum as part of a header.

For more information, see [the `binbin` documentation](https://docs.rs/binbin).
