use enum_dispatch::enum_dispatch;

pub(crate) const BUF_CAP: usize = 4096;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(&self) -> Vec<u8>;
}
