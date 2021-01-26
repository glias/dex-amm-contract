use ckb_tool::ckb_types::core::Capacity;
use ckb_tool::ckb_types::packed::Uint128;
use ckb_tool::ckb_types::{bytes::Bytes, prelude::*};

use crate::schema::cell::InfoCellData;

pub struct InfoCell {
    pub capacity: Capacity,
    pub data:     Bytes,
}

impl InfoCell {
    pub fn builder() -> InfoCellBuilder {
        InfoCellBuilder::default()
    }
}

#[derive(Default)]
pub struct InfoCellBuilder {
    capacity:                 u64,
    ckb_reserve:              u128,
    sudt_reserve:             u128,
    total_liquidity:          u128,
    liquidity_sudt_type_hash: [u8; 32],
}

impl InfoCellBuilder {
    pub fn capacity(mut self, capacity: u64) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn ckb_reserve(mut self, ckb_reserve: u128) -> Self {
        self.ckb_reserve = ckb_reserve;
        self
    }

    pub fn sudt_reserve(mut self, sudt_reserve: u128) -> Self {
        self.sudt_reserve = sudt_reserve;
        self
    }

    pub fn liquidity_sudt_type_hash(mut self, liquidity_sudt_type_hash: [u8; 32]) -> Self {
        self.liquidity_sudt_type_hash = liquidity_sudt_type_hash;
        self
    }

    pub fn build(self) -> InfoCell {
        let info_data = InfoCellData::new_builder()
            .sudt_reserve(self.sudt_reserve.pack())
            .ckb_reserve(self.ckb_reserve.pack())
            .total_liquidity(self.total_liquidity.pack())
            .liquidity_sudt_type_hash(self.liquidity_sudt_type_hash.pack())
            .build();

        InfoCell {
            capacity: Capacity::shannons(self.capacity),
            data:     info_data.as_bytes(),
        }
    }
}

pub struct SudtCell {
    pub capacity: Capacity,
    pub data:     Bytes,
}

impl SudtCell {
    pub fn new(capacity: u64, amount: u128) -> Self {
        let sudt_data: Uint128 = amount.pack();

        SudtCell {
            capacity: Capacity::shannons(capacity),
            data:     sudt_data.as_bytes(),
        }
    }
}

pub struct FreeCell {
    pub capacity: Capacity,
    pub data:     Bytes,
}

impl FreeCell {
    pub fn new(capacity: u64) -> Self {
        FreeCell {
            capacity: Capacity::shannons(capacity),
            data:     Bytes::new(),
        }
    }
}
