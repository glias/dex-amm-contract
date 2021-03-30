pub mod generated;
pub use generated::basic;
pub use generated::cell;

use ckb_tool::ckb_types::{bytes::Bytes, prelude::*};

impl Pack<basic::Uint128> for u128 {
    fn pack(&self) -> basic::Uint128 {
        basic::Uint128::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl Pack<basic::Uint64> for u64 {
    fn pack(&self) -> basic::Uint64 {
        basic::Uint64::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}

impl Pack<basic::Byte32> for [u8; 32] {
    fn pack(&self) -> basic::Byte32 {
        let tmp = Bytes::from(self.to_vec());
        basic::Byte32::new_unchecked(tmp)
    }
}

impl Pack<basic::Uint8> for u8 {
    fn pack(&self) -> basic::Uint8 {
        basic::Uint8::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
    }
}
