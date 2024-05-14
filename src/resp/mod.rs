use std::collections::BTreeMap;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};

use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use thiserror::Error;

mod decode;
mod encode;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(&self) -> Vec<u8>;
}

pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;

    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

#[derive(Debug, Error, PartialEq)]
pub enum RespError {
    #[error("Invalid frame : {0}")]
    InvalidFrame(String),
    #[error("Invalid frame type : {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length : {0}")]
    InvalidFrameLength(isize),
    #[error("Not complete")]
    NotComplete,

    #[error("parse int error : {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("parse float error : {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("UTF8 error : {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    Null(RespNull),
    NullArray(RespNullArray),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleString(pub(crate) String);

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleError(pub(crate) String);

#[derive(Debug, Clone, PartialEq)]
pub struct BulkString(pub(crate) Vec<u8>);

#[derive(Debug, Clone, PartialEq)]
pub struct RespNull;

#[derive(Debug, Clone, PartialEq)]
pub struct RespNullArray;

#[derive(Debug, Clone, PartialEq)]
pub struct RespNullBulkString;

#[derive(Debug, Clone, PartialEq)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

#[derive(Debug, Clone, PartialEq)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);

#[derive(Debug, Clone, PartialEq)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl RespNullBulkString {
    pub fn new() -> Self {
        RespNullBulkString
    }
}

impl Default for RespNullBulkString {
    fn default() -> Self {
        Self::new()
    }
}
impl RespArray {
    pub fn new(v: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(v.into())
    }
}

impl RespMap {
    // pub fn new(m: impl Into<BTreeMap<String, RespFrame>>) -> Self {
    //     RespMap(m.into())
    // }

    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        let s = s.into();

        RespSet(s)
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}
impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}
impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}
impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
