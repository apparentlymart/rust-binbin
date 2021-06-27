pub trait Endian {}

pub enum LittleEndian {}

impl Endian for LittleEndian {}

pub enum BigEndian {}

impl Endian for BigEndian {}
