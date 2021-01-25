use core::cmp::{Eq, PartialEq};
use core::convert::TryFrom;
use core::result::Result;

use ckb_std::error::SysError as Error;

use crate::{check_args_len, decode_u128, decode_u64, decode_u8};

const LIQUIDITY_ORDER_ARGS_LEN: usize = 113;
const SWAP_ORDER_ARGS_LEN: usize = 105;
const INFO_CELL_DATA_LEN: usize = 80;
const SUDT_AMOUNT_DATA_LEN: usize = 16;

#[derive(Debug, PartialEq, Eq)]
pub enum OrderKind {
    SellCKB,
    BuyCKB,
}

impl TryFrom<u8> for OrderKind {
    type Error = Error;

    fn try_from(input: u8) -> Result<OrderKind, Error> {
        match input {
            0 => Ok(OrderKind::SellCKB),
            1 => Ok(OrderKind::BuyCKB),
            _ => Err(Error::Encoding),
        }
    }
}

impl Into<u8> for OrderKind {
    fn into(self) -> u8 {
        match self {
            OrderKind::SellCKB => 0,
            OrderKind::BuyCKB => 1,
        }
    }
}

#[derive(Debug)]
pub struct LiquidityRequestLockArgs {
    pub user_lock_hash: [u8; 32],
    pub version:        u8,
    pub amount_0:       u64,
    pub amount_1:       u128,
    pub info_type_hash: [u8; 32],
    pub tips:           u128,
    pub tips_sudt:      u128,
}

impl LiquidityRequestLockArgs {
    pub fn from_raw(cell_raw_data: &[u8]) -> Result<Self, Error> {
        check_args_len(cell_raw_data.len(), LIQUIDITY_ORDER_ARGS_LEN)?;

        let mut user_lock_hash = [0u8; 32];
        user_lock_hash.copy_from_slice(&cell_raw_data[0..32]);
        let version = decode_u8(&cell_raw_data[32..33])?;
        let amount_0 = decode_u64(&cell_raw_data[49..57])?;
        let amount_1 = decode_u128(&cell_raw_data[33..49])?;
        let mut info_type_hash = [0u8; 32];
        info_type_hash.copy_from_slice(&cell_raw_data[57..89]);
        let tips = decode_u128(&cell_raw_data[89..97])?;
        let tips_sudt = decode_u128(&cell_raw_data[97..113])?;

        Ok(LiquidityRequestLockArgs {
            user_lock_hash,
            version,
            amount_0,
            amount_1,
            info_type_hash,
            tips,
            tips_sudt,
        })
    }
}

#[derive(Debug)]
pub struct SwapRequestLockArgs {
    pub user_lock_hash: [u8; 32],
    pub version:        u8,
    pub min_amount_out: u128,
    pub sudt_type_hash: [u8; 32],
    pub tips:           u64,
    pub tips_sudt:      u128,
}

impl SwapRequestLockArgs {
    pub fn from_raw(cell_raw_data: &[u8]) -> Result<Self, Error> {
        check_args_len(cell_raw_data.len(), SWAP_ORDER_ARGS_LEN)?;

        let mut user_lock_hash = [0u8; 32];
        user_lock_hash.copy_from_slice(&cell_raw_data[0..32]);
        let version = decode_u8(&cell_raw_data[32..33])?;
        let min_amount_out = decode_u128(&cell_raw_data[33..49])?;
        let mut sudt_type_hash = [0u8; 32];
        sudt_type_hash.copy_from_slice(&cell_raw_data[49..81]);
        let tips = decode_u64(&cell_raw_data[81..89])?;
        let tips_sudt = decode_u128(&cell_raw_data[89..105])?;

        Ok(SwapRequestLockArgs {
            user_lock_hash,
            version,
            min_amount_out,
            sudt_type_hash,
            tips,
            tips_sudt,
        })
    }
}

#[derive(Debug)]
pub struct InfoCellData {
    pub ckb_reserve:              u128,
    pub sudt_reserve:             u128,
    pub total_liquidity:          u128,
    pub liquidity_sudt_type_hash: [u8; 32],
}

impl InfoCellData {
    pub fn from_raw(cell_raw_data: &[u8]) -> Result<InfoCellData, Error> {
        check_args_len(cell_raw_data.len(), INFO_CELL_DATA_LEN)?;

        let ckb_reserve = decode_u128(&cell_raw_data[..16])?;
        let sudt_reserve = decode_u128(&cell_raw_data[16..32])?;
        let total_liquidity = decode_u128(&cell_raw_data[32..48])?;
        let mut liquidity_sudt_type_hash = [0u8; 32];
        liquidity_sudt_type_hash.copy_from_slice(&cell_raw_data[48..68]);

        Ok(InfoCellData {
            ckb_reserve,
            sudt_reserve,
            total_liquidity,
            liquidity_sudt_type_hash,
        })
    }
}

#[derive(Debug)]
pub struct SUDTAmountData {
    pub sudt_amount: u128,
}

impl SUDTAmountData {
    pub fn from_raw(cell_raw_data: &[u8]) -> Result<Self, Error> {
        check_args_len(cell_raw_data.len(), SUDT_AMOUNT_DATA_LEN)?;
        let sudt_amount = decode_u128(&cell_raw_data[..16])?;

        Ok(SUDTAmountData { sudt_amount })
    }
}
