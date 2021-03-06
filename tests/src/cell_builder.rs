use ckb_tool::ckb_types::core::Capacity;
use ckb_tool::ckb_types::packed::{OutPoint, Uint128};
use ckb_tool::ckb_types::{bytes::Bytes, prelude::*};

use crate::schema::cell::{InfoCellData, LiquidityRequestLockArgs, SwapRequestLockArgs};

pub struct InfoCell {
    pub capacity: Capacity,
    pub data:     Bytes,
}

impl InfoCell {
    pub fn new_unchecked(capacity: u64, data: Bytes) -> Self {
        InfoCell {
            capacity: Capacity::shannons(capacity),
            data,
        }
    }
}

#[derive(Default)]
pub struct LiquidityRequestCell {
    pub capacity: Capacity,
    pub data:     Bytes,
}

impl LiquidityRequestCell {
    pub fn new(capacity: u64, amount: u128) -> Self {
        let sudt_data: Uint128 = amount.pack();

        LiquidityRequestCell {
            capacity: Capacity::shannons(capacity),
            data:     sudt_data.as_bytes(),
        }
    }

    pub fn new_unchecked(capacity: u64, data: Bytes) -> Self {
        LiquidityRequestCell {
            capacity: Capacity::shannons(capacity),
            data,
        }
    }
}

#[derive(Default)]
pub struct SwapRequestCell {
    pub capacity: Capacity,
    pub data:     Bytes,
    pub is_sudt:  bool,
}

impl SwapRequestCell {
    pub fn new_ckb(capacity: u64) -> Self {
        SwapRequestCell {
            capacity: Capacity::shannons(capacity),
            data:     Bytes::default(),
            is_sudt:  false,
        }
    }

    pub fn new_sudt(capacity: u64, amount: u128) -> Self {
        let sudt_data: Uint128 = amount.pack();

        SwapRequestCell {
            capacity: Capacity::shannons(capacity),
            data:     sudt_data.as_bytes(),
            is_sudt:  true,
        }
    }

    pub fn new_unchecked(capacity: u64, data: Bytes, is_sudt: bool) -> Self {
        SwapRequestCell {
            capacity: Capacity::shannons(capacity),
            data,
            is_sudt,
        }
    }
}

#[derive(Default)]
pub struct LiquidityRequestLockArgsBuilder {
    info_type_hash: [u8; 32],
    user_lock_hash: [u8; 32],
    version:        u8,
    sudt_min:       u128,
    ckb_min:        u64,
    tips:           u64,
    tips_sudt:      u128,
}

impl LiquidityRequestLockArgsBuilder {
    pub fn user_lock_hash(mut self, user_lock_hash: [u8; 32]) -> Self {
        self.user_lock_hash = user_lock_hash;
        self
    }

    pub fn version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    pub fn sudt_min(mut self, sudt_min: u128) -> Self {
        self.sudt_min = sudt_min;
        self
    }

    pub fn ckb_min(mut self, ckb_min: u64) -> Self {
        self.ckb_min = ckb_min;
        self
    }

    pub fn info_type_hash(mut self, info_type_hash: [u8; 32]) -> Self {
        self.info_type_hash = info_type_hash;
        self
    }

    pub fn tips(mut self, tips: u64) -> Self {
        self.tips = tips;
        self
    }

    pub fn tips_sudt(mut self, tips_sudt: u128) -> Self {
        self.tips_sudt = tips_sudt;
        self
    }

    pub fn build(self) -> LiquidityRequestLockArgs {
        LiquidityRequestLockArgs::new_builder()
            .info_type_hash(self.info_type_hash.pack())
            .user_lock_hash(self.user_lock_hash.pack())
            .version(self.version.pack())
            .sudt_min(self.sudt_min.pack())
            .ckb_min(self.ckb_min.pack())
            .tips(self.tips.pack())
            .tips_sudt(self.tips_sudt.pack())
            .build()
    }
}

#[derive(Default)]
pub struct SwapRequestLockArgsBuilder {
    sudt_type_hash: [u8; 32],
    user_lock_hash: [u8; 32],
    version:        u8,
    amount_out_min: u128,
    tips:           u64,
    tips_sudt:      u128,
}

impl SwapRequestLockArgsBuilder {
    pub fn user_lock_hash(mut self, user_lock_hash: [u8; 32]) -> Self {
        self.user_lock_hash = user_lock_hash;
        self
    }

    pub fn version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    pub fn amount_out_min(mut self, amount_out_min: u128) -> Self {
        self.amount_out_min = amount_out_min;
        self
    }

    pub fn sudt_type_hash(mut self, sudt_type_hash: [u8; 32]) -> Self {
        self.sudt_type_hash = sudt_type_hash;
        self
    }

    pub fn tips(mut self, tips: u64) -> Self {
        self.tips = tips;
        self
    }

    pub fn tips_sudt(mut self, tips_sudt: u128) -> Self {
        self.tips_sudt = tips_sudt;
        self
    }

    pub fn build(self) -> SwapRequestLockArgs {
        SwapRequestLockArgs::new_builder()
            .sudt_type_hash(self.sudt_type_hash.pack())
            .user_lock_hash(self.user_lock_hash.pack())
            .version(self.version.pack())
            .amount_out_min(self.amount_out_min.pack())
            .tips(self.tips.pack())
            .tips_sudt(self.tips_sudt.pack())
            .build()
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

    pub fn total_liquidity(mut self, total_liquidity: u128) -> Self {
        self.total_liquidity = total_liquidity;
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
    pub capacity:  Capacity,
    pub data:      Bytes,
    pub out_point: Option<OutPoint>,
}

impl SudtCell {
    pub fn new(capacity: u64, amount: u128) -> Self {
        let sudt_data: Uint128 = amount.pack();

        SudtCell {
            capacity:  Capacity::shannons(capacity),
            data:      sudt_data.as_bytes(),
            out_point: None,
        }
    }

    pub fn new_with_out_point(capacity: u64, amount: u128, out_point: OutPoint) -> Self {
        let sudt_data: Uint128 = amount.pack();

        SudtCell {
            capacity:  Capacity::shannons(capacity),
            data:      sudt_data.as_bytes(),
            out_point: Some(out_point),
        }
    }

    pub fn new_unchecked(capacity: u64, data: Bytes) -> Self {
        SudtCell {
            capacity: Capacity::shannons(capacity),
            data,
            out_point: None,
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
