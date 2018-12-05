//! FBX data.

use fbxcel::parser::binary as fbxbin;

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
    ///
    /// Note that the string value in FBX might be non-UTF-8.
    String(Result<String, Vec<u8>>),
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
            Attribute::String(Ok(ref val)) => {
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
            Attribute::String(Err(ref arr)) | Attribute::Binary(ref arr) => arr
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

    /// Reads attribute in parser event into `Attribute`.
    pub fn read<R: fbxbin::ParserSource>(attr: fbxbin::Attribute<R>) -> fbxbin::Result<Self> {
        use fbxcel::parser::binary::{ArrayAttribute, PrimitiveAttribute, SpecialAttributeType};

        match attr {
            fbxbin::Attribute::Primitive(PrimitiveAttribute::Bool(val)) => {
                Ok(Attribute::SingleBool(val))
            }
            fbxbin::Attribute::Primitive(PrimitiveAttribute::I16(val)) => {
                Ok(Attribute::SingleI16(val))
            }
            fbxbin::Attribute::Primitive(PrimitiveAttribute::I32(val)) => {
                Ok(Attribute::SingleI32(val))
            }
            fbxbin::Attribute::Primitive(PrimitiveAttribute::I64(val)) => {
                Ok(Attribute::SingleI64(val))
            }
            fbxbin::Attribute::Primitive(PrimitiveAttribute::F32(val)) => {
                Ok(Attribute::SingleF32(val))
            }
            fbxbin::Attribute::Primitive(PrimitiveAttribute::F64(val)) => {
                Ok(Attribute::SingleF64(val))
            }
            fbxbin::Attribute::Array(ArrayAttribute::Bool(arr)) => {
                Ok(Attribute::ArrayBool(arr.into_vec()?))
            }
            fbxbin::Attribute::Array(ArrayAttribute::I32(arr)) => {
                Ok(Attribute::ArrayI32(arr.into_vec()?))
            }
            fbxbin::Attribute::Array(ArrayAttribute::I64(arr)) => {
                Ok(Attribute::ArrayI64(arr.into_vec()?))
            }
            fbxbin::Attribute::Array(ArrayAttribute::F32(arr)) => {
                Ok(Attribute::ArrayF32(arr.into_vec()?))
            }
            fbxbin::Attribute::Array(ArrayAttribute::F64(arr)) => {
                Ok(Attribute::ArrayF64(arr.into_vec()?))
            }
            fbxbin::Attribute::Special(reader) => match reader.value_type() {
                SpecialAttributeType::Binary => Ok(Attribute::Binary(reader.into_vec()?)),
                SpecialAttributeType::String => Ok(Attribute::String({
                    String::from_utf8(reader.into_vec()?).map_err(|err| err.into_bytes())
                })),
            },
        }
    }
}
