use std::io::{self, Write};
use byteorder::{BigEndian, WriteBytesExt};
use crate::address::Address;
use crate::objects::amount::Amount;

pub struct BinaryFormat<'a, T>(pub &'a T);

/// TODO: Remove `pub`?
pub struct BinaryFormatWithoutFieldUid<'a, T>(pub &'a T);

/// Serialization of this should, contrary to intuition, be defined only for formats that are serialized with length.
///
/// TODO: Remove `pub`?
pub struct BinaryFormatWithoutLength<'a, T>(pub &'a T);

// TODO: Make it asynchronous.
pub trait Serialize {
    fn serialize(&self, writer: &mut dyn Write) -> io::Result<()>;
}

pub struct XrplType {
    pub type_code: i16,
}

pub struct XrplBinaryField<'a, T> {
    pub xrpl_type: &'a XrplType,
    pub field_code: i16,
    pub value: &'a T,
}

impl<'a, T> XrplBinaryField<'a, T> {
    pub fn type_code(&self) -> i16 {
        self.xrpl_type.type_code
    }
}

impl<'a, T> XrplBinaryField<'a, T> {
    pub fn field_uid(&self) -> Vec<u8> {
        // https://xrpl.org/serialization.html#field-ids
        match (self.type_code() >= 16, self.field_code >= 16) {
            (false, false) => vec![(self.type_code() << 4 | self.field_code) as u8],
            (true, false) => vec![self.field_code as u8, self.type_code() as u8],
            (false, true) => vec![(self.type_code() << 4) as u8, self.field_code as u8],
            (true, true) => vec![0, self.type_code() as u8, self.field_code as u8],
        }
    }
    pub fn serialize_field_uid(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.field_uid())
    }
}

// TODO: `pub`?
pub fn serialize_length(writer: &mut dyn Write, length: usize) -> io::Result<()> {
    // Essentially copied from https://github.com/gmosx/xrpl_sdk_rust/blob/1ba1c8872caa1a2f80db4346f685e8c940518bc9/xrpl_binary_codec/src/serializer.rs:
    if length <= 192 {
        writer.write_u8(length as u8)?;
    } else if length <= 12480 {
        let length = length - 192;
        writer.write_u8(193 + (length >> 8) as u8)?;
        writer.write_u8((length & 0xff) as u8)?;
    } else if length <= 918744 {
        let length = length - 12481;
        writer.write_u8(241 + (length >> 16) as u8)?;
        writer.write_u8(((length >> 8) & 0xff) as u8)?;
        writer.write_u8((length & 0xff) as u8)?;
    } else {
        panic!("too long data"); // TODO: better error than panic?
    }
    Ok(())
}

impl<'a, T> Serialize for XrplBinaryField<'a, T>
    where BinaryFormatWithoutFieldUid<'a, T>: Serialize
{
    fn serialize(&self, writer: &mut dyn Write) -> io::Result<()> {
        self.serialize_field_uid(writer)?;
        BinaryFormatWithoutFieldUid::<T>(self.value).serialize(writer)
    }
}

impl<'a, T> Serialize for BinaryFormatWithoutFieldUid<'a, T>
    where BinaryFormatWithoutLength<'a, T>: Serialize
{
    fn serialize(&self, writer: &mut dyn Write) -> io::Result<()> {
        let mut buf: Vec<u8> = Vec::new();
        BinaryFormatWithoutLength::<T>(self.0).serialize(&mut buf)?;
        serialize_length(writer, buf.len())?;
        writer.write_all(&buf)
    }
}

impl<'a> Serialize for BinaryFormatWithoutLength<'a, u32> {
    fn serialize(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_u32::<BigEndian>(*self.0)
    }
}

// Copied from xrpl_sdk_rust
fn write_currency(writer: &mut dyn Write, currency: &str) -> io::Result<()> {
    // Non-standard currency codes are 160 bits = 20 bytes in hex (40 chars).

    if currency.len() == 40 {
        // Non-standard currency code.
        let currency_bytes = hex::decode(currency).unwrap();
        // if currency_bytes[0] == 0x00 {
        writer.write_all(&currency_bytes)?;
        return Ok(());
        // }
    }

    // Standard currency code.

    // 8 bits
    writer.write_u8(0x00)?;

    // 88 bits
    for _ in 0..11 {
        writer.write_u8(0x00)?;
    }

    // 24 bits
    writer.write_all(currency.as_bytes())?;

    // 40 bits
    for _ in 0..5 {
        writer.write_u8(0x00)?;
    }

    Ok(())
}

impl<'a> Serialize for BinaryFormatWithoutLength<'a, Address> {
    fn serialize(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.0.0.0)
    }
}

impl<'a> Serialize for BinaryFormatWithoutFieldUid<'a, Amount> {
    fn serialize(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_f64::<BigEndian>(self.0.value)?;
        write_currency(writer, &self.0.currency)?;
        BinaryFormatWithoutLength(&self.0.issuer).serialize(writer)
    }
}