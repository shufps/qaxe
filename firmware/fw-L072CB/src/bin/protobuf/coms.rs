// Automatically generated rust module for 'coms.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use alloc::vec::Vec;
use alloc::borrow::Cow;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QRequest<'a> {
    pub id: i32,
    pub op: i32,
    pub data: Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for QRequest<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = r.read_int32(bytes)?,
                Ok(16) => msg.op = r.read_int32(bytes)?,
                Ok(26) => msg.data = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for QRequest<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.id == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.id) as u64) }
        + if self.op == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.op) as u64) }
        + if self.data == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.data).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.id != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.id))?; }
        if self.op != 0i32 { w.write_with_tag(16, |w| w.write_int32(*&self.op))?; }
        if self.data != Cow::Borrowed(b"") { w.write_with_tag(26, |w| w.write_bytes(&**&self.data))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QResponse<'a> {
    pub id: i32,
    pub error: i32,
    pub data: Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for QResponse<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = r.read_int32(bytes)?,
                Ok(16) => msg.error = r.read_int32(bytes)?,
                Ok(26) => msg.data = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for QResponse<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.id == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.id) as u64) }
        + if self.error == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.error) as u64) }
        + if self.data == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.data).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.id != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.id))?; }
        if self.error != 0i32 { w.write_with_tag(16, |w| w.write_int32(*&self.error))?; }
        if self.data != Cow::Borrowed(b"") { w.write_with_tag(26, |w| w.write_bytes(&**&self.data))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QControl {
    pub state_1v2: i32,
    pub pwm1: i32,
    pub pwm2: i32,
}

impl<'a> MessageRead<'a> for QControl {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.state_1v2 = r.read_int32(bytes)?,
                Ok(16) => msg.pwm1 = r.read_int32(bytes)?,
                Ok(24) => msg.pwm2 = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for QControl {
    fn get_size(&self) -> usize {
        0
        + if self.state_1v2 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.state_1v2) as u64) }
        + if self.pwm1 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.pwm1) as u64) }
        + if self.pwm2 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.pwm2) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.state_1v2 != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.state_1v2))?; }
        if self.pwm1 != 0i32 { w.write_with_tag(16, |w| w.write_int32(*&self.pwm1))?; }
        if self.pwm2 != 0i32 { w.write_with_tag(24, |w| w.write_int32(*&self.pwm2))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QState {
    pub pgood_1v2: i32,
    pub temp1: i32,
    pub temp2: i32,
}

impl<'a> MessageRead<'a> for QState {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.pgood_1v2 = r.read_int32(bytes)?,
                Ok(16) => msg.temp1 = r.read_int32(bytes)?,
                Ok(24) => msg.temp2 = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for QState {
    fn get_size(&self) -> usize {
        0
        + if self.pgood_1v2 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.pgood_1v2) as u64) }
        + if self.temp1 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.temp1) as u64) }
        + if self.temp2 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.temp2) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.pgood_1v2 != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.pgood_1v2))?; }
        if self.temp1 != 0i32 { w.write_with_tag(16, |w| w.write_int32(*&self.temp1))?; }
        if self.temp2 != 0i32 { w.write_with_tag(24, |w| w.write_int32(*&self.temp2))?; }
        Ok(())
    }
}

