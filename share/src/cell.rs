use core::cmp::{Eq, PartialEq};
use core::convert::TryFrom;
use core::result::Result;

use crate::{check_args_len, decode_u128, decode_u8};

use ckb_std::ckb_types::bytes::Bytes;
use ckb_std::error::SysError as Error;

const LIQUIDITY_ORDER_ARGS_LEN: usize = 86;
const SWAP_ORDER_ARGS_LEN: usize = 67;
const INFO_CELL_DATA_LEN: usize = 68;
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
pub struct LiquidityOrderLockArgs {
    pub user_lock_hash: Bytes,
    pub version:        u8,
    pub amount_0:       u128,
    pub amount_1:       u128,
    pub info_type_hash: Bytes,
}

impl LiquidityOrderLockArgs {
    pub fn from_raw(cell_raw_data: &[u8]) -> Result<Self, Error> {
        check_args_len(cell_raw_data.len(), LIQUIDITY_ORDER_ARGS_LEN)?;

        let mut buf = [0u8; 32];
        buf.copy_from_slice(&cell_raw_data[0..32]);
        let user_lock_hash = Bytes::from(buf.to_vec());
        let version = decode_u8(&cell_raw_data[32..33])?;
        let amount_0 = decode_u128(&cell_raw_data[33..49])?;
        let amount_1 = decode_u128(&cell_raw_data[49..65])?;
        let mut buf = [0u8; 20];
        buf.copy_from_slice(&cell_raw_data[65..85]);
        let info_type_hash = Bytes::from(buf.to_vec());

        Ok(LiquidityOrderLockArgs {
            user_lock_hash,
            version,
            amount_0,
            amount_1,
            info_type_hash,
        })
    }
}

#[derive(Debug)]
pub struct SwapOrderLockArgs {
    pub user_lock_hash: Bytes,
    pub version:        u8,
    pub amount_in:      u128,
    pub min_amount_out: u128,
    pub kind:           OrderKind,
}

impl SwapOrderLockArgs {
    pub fn from_raw(cell_raw_data: &[u8]) -> Result<Self, Error> {
        check_args_len(cell_raw_data.len(), SWAP_ORDER_ARGS_LEN)?;

        let mut buf = [0u8; 32];
        buf.copy_from_slice(&cell_raw_data[0..32]);
        let user_lock_hash = Bytes::from(buf.to_vec());
        let version = decode_u8(&cell_raw_data[32..33])?;
        let amount_in = decode_u128(&cell_raw_data[33..49])?;
        let min_amount_out = decode_u128(&cell_raw_data[49..65])?;
        let kind = OrderKind::try_from(decode_u8(&cell_raw_data[65..66])?)?;

        Ok(SwapOrderLockArgs {
            user_lock_hash,
            version,
            amount_in,
            min_amount_out,
            kind,
        })
    }
}

#[derive(Debug)]
pub struct InfoCellData {
    pub ckb_reserve:              u128,
    pub sudt_reserve:             u128,
    pub total_liquidity:          u128,
    pub liquidity_sudt_type_hash: [u8; 20],
}

impl InfoCellData {
    pub fn from_raw(cell_raw_data: &[u8]) -> Result<InfoCellData, Error> {
        check_args_len(cell_raw_data.len(), INFO_CELL_DATA_LEN)?;

        let ckb_reserve = decode_u128(&cell_raw_data[..16])?;
        let sudt_reserve = decode_u128(&cell_raw_data[16..32])?;
        let total_liquidity = decode_u128(&cell_raw_data[32..48])?;
        let mut liquidity_sudt_type_hash = [0u8; 20];
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
