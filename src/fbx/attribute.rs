//! FBX node attribute.

use std::io;

use fbxcel::pull_parser::{self as fbxbin, Result};

/// FBX node attribute.
#[derive(Debug, Clone)]
pub enum Attribute {
    /// `bool`.
    SingleBool(bool),
    /// `i16`.
    SingleI16(i16),
    /// `i32`.
    SingleI32(i32),
    /// `i64`.
    SingleI64(i64),
    /// `f32`.
    SingleF32(f32),
    /// `f64`.
    SingleF64(f64),
    /// `[bool]`.
    ArrayBool(Vec<bool>),
    /// `[i32]`.
    ArrayI32(Vec<i32>),
    /// `[i64]`.
    ArrayI64(Vec<i64>),
    /// `[f32]`.
    ArrayF32(Vec<f32>),
    /// `[f64]`.
    ArrayF64(Vec<f64>),
    /// `String`.
    String(String),
    /// `[u8]`.
    Binary(Vec<u8>),
}

impl Attribute {
    /// Returns type name.
    pub fn type_string(&self) -> &str {
        match *self {
            Attribute::SingleBool(_) => "bool",
            Attribute::SingleI16(_) => "i16",
            Attribute::SingleI32(_) => "i32",
            Attribute::SingleI64(_) => "i64",
            Attribute::SingleF32(_) => "f32",
            Attribute::SingleF64(_) => "f64",
            Attribute::ArrayBool(_) => "[bool]",
            Attribute::ArrayI32(_) => "[i32]",
            Attribute::ArrayI64(_) => "[i64]",
            Attribute::ArrayF32(_) => "[f32]",
            Attribute::ArrayF64(_) => "[f64]",
            Attribute::String(_) => "String",
            Attribute::Binary(_) => "[u8]",
        }
    }

    /// Returns string representation.
    pub fn value_string(&self) -> String {
        match *self {
            Attribute::SingleBool(val) => val.to_string(),
            Attribute::SingleI16(val) => val.to_string(),
            Attribute::SingleI32(val) => val.to_string(),
            Attribute::SingleI64(val) => val.to_string(),
            Attribute::SingleF32(val) => val.to_string(),
            Attribute::SingleF64(val) => val.to_string(),
            Attribute::ArrayBool(ref arr) => arr
                .iter()
                .enumerate()
                .map(|(i, &val)| match (i & 0x0f == 0x0f, val) {
                    (false, false) => "0, ",
                    (false, true) => "1, ",
                    (true, false) => "0,\n",
                    (true, true) => "1,\n",
                })
                .collect(),
            Attribute::ArrayI32(ref arr) => arr
                .iter()
                .enumerate()
                .map(|(i, &val)| {
                    if i & 0x0f == 0x0f {
                        format!("{},\n", val)
                    } else {
                        format!("{}, ", val)
                    }
                })
                .collect(),
            Attribute::ArrayI64(ref arr) => arr
                .iter()
                .enumerate()
                .map(|(i, &val)| {
                    if i & 0x0f == 0x0f {
                        format!("{},\n", val)
                    } else {
                        format!("{}, ", val)
                    }
                })
                .collect(),
            Attribute::ArrayF32(ref arr) => arr
                .iter()
                .enumerate()
                .map(|(i, &val)| {
                    if i & 0x0f == 0x0f {
                        format!("{},\n", val)
                    } else {
                        format!("{}, ", val)
                    }
                })
                .collect(),
            Attribute::ArrayF64(ref arr) => arr
                .iter()
                .enumerate()
                .map(|(i, &val)| {
                    if i & 0x0f == 0x0f {
                        format!("{},\n", val)
                    } else {
                        format!("{}, ", val)
                    }
                })
                .collect(),
            Attribute::String(ref val) => {
                val.chars()
                    .fold(String::with_capacity(val.len()), |mut s, c| {
                        match c {
                            '\n' | '\t' => s.push(c),
                            '\r' => s.push_str("\\r"),
                            _ if (c <= '\x1f') || (c == '\x7f') => {
                                s.push_str(&format!("\\x{:02x}", c as u32))
                            }
                            c => s.push(c),
                        }
                        s
                    })
            }
            Attribute::Binary(ref arr) => arr
                .iter()
                .enumerate()
                .map(|(i, &val)| {
                    if i & 0x0f == 0x0f {
                        format!("{:02x},\n", val)
                    } else {
                        format!("{:02x}, ", val)
                    }
                })
                .collect(),
        }
    }
}

/// FBX 7.4 attribute loader.
#[derive(Debug, Clone)]
pub struct AttributeLoader;

impl fbxbin::v7400::LoadAttribute for AttributeLoader {
    type Output = Attribute;

    fn expecting(&self) -> String {
        "any attributes".to_owned()
    }

    fn load_bool(self, v: bool) -> Result<Self::Output> {
        Ok(Attribute::SingleBool(v))
    }
    fn load_i16(self, v: i16) -> Result<Self::Output> {
        Ok(Attribute::SingleI16(v))
    }
    fn load_i32(self, v: i32) -> Result<Self::Output> {
        Ok(Attribute::SingleI32(v))
    }
    fn load_i64(self, v: i64) -> Result<Self::Output> {
        Ok(Attribute::SingleI64(v))
    }
    fn load_f32(self, v: f32) -> Result<Self::Output> {
        Ok(Attribute::SingleF32(v))
    }
    fn load_f64(self, v: f64) -> Result<Self::Output> {
        Ok(Attribute::SingleF64(v))
    }
    fn load_seq_bool(
        self,
        iter: impl Iterator<Item = Result<bool>>,
        _: usize,
    ) -> Result<Self::Output> {
        iter.collect::<Result<_>>().map(Attribute::ArrayBool)
    }
    fn load_seq_i32(
        self,
        iter: impl Iterator<Item = Result<i32>>,
        _: usize,
    ) -> Result<Self::Output> {
        iter.collect::<Result<_>>().map(Attribute::ArrayI32)
    }
    fn load_seq_i64(
        self,
        iter: impl Iterator<Item = Result<i64>>,
        _: usize,
    ) -> Result<Self::Output> {
        iter.collect::<Result<_>>().map(Attribute::ArrayI64)
    }
    fn load_seq_f32(
        self,
        iter: impl Iterator<Item = Result<f32>>,
        _: usize,
    ) -> Result<Self::Output> {
        iter.collect::<Result<_>>().map(Attribute::ArrayF32)
    }
    fn load_seq_f64(
        self,
        iter: impl Iterator<Item = Result<f64>>,
        _: usize,
    ) -> Result<Self::Output> {
        iter.collect::<Result<_>>().map(Attribute::ArrayF64)
    }
    fn load_binary(self, mut reader: impl io::Read, len: u64) -> Result<Self::Output> {
        let mut buf = Vec::with_capacity(len as usize);
        reader.read_to_end(&mut buf)?;
        Ok(Attribute::Binary(buf))
    }
    fn load_string(self, mut reader: impl io::Read, len: u64) -> Result<Self::Output> {
        let mut buf = String::with_capacity(len as usize);
        reader.read_to_string(&mut buf)?;
        Ok(Attribute::String(buf))
    }
}
