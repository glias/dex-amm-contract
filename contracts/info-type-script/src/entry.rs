mod type_id;
mod verify;

use alloc::vec::Vec;
use core::result::Result;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use share::cell::LiquidityRequestLockArgs;
use share::cell::{InfoCellData, SUDTAmountData};
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        packed::{Byte, CellOutput},
        prelude::*,
    },
    default_alloc,
    // debug,
    high_level::{
        load_cell, load_cell_data, load_cell_lock, load_cell_lock_hash, load_script, QueryIter,
    },
};
use share::{blake2b, get_cell_type_hash};
use type_id::verify_type_id;

use crate::error::Error;

const POOL_BASE_CAPACITY: u128 = 18_600_000_000;

pub static INFO_LOCK_CODE_HASH: &str =
    include!(concat!(env!("OUT_DIR"), "/info_lock_code_hash.rs"));

// Alloc 4K fast HEAP + 2M HEAP to receives PrefilledData
default_alloc!(4 * 1024, 2048 * 1024, 64);

pub fn main() -> Result<(), Error> {
    let info_type_code_hash = load_script()?.code_hash().unpack();

    let input_info_cell_count = QueryIter::new(load_cell, Source::Input)
        .filter(|cell| {
            cell.type_().to_opt().map_or_else(
                || false,
                |script| script.code_hash().unpack() == info_type_code_hash,
            )
        })
        .count();
    let output_info_cell_count = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| {
            cell.type_().to_opt().map_or_else(
                || false,
                |script| script.code_hash().unpack() == info_type_code_hash,
            )
        })
        .count();

    if input_info_cell_count == 0 && output_info_cell_count == 1 {
        verify_info_creation(&load_cell(0, Source::Output)?)?;
        return Ok(());
    }

    if input_info_cell_count != 1 || output_info_cell_count != 1 {
        return Err(Error::MoreThanOneLiquidityPool);
    }

    let info_in_data = InfoCellData::from_raw(&load_cell_data(0, Source::Input)?)?;
    let pool_in_cell = load_cell(1, Source::Input)?;
    let pool_in_data = SUDTAmountData::from_raw(&load_cell_data(1, Source::Input)?)?;

    if (pool_in_cell.capacity().unpack() as u128) != POOL_BASE_CAPACITY + info_in_data.ckb_reserve {
        return Err(Error::CKBReserveAmountDiff);
    }

    if pool_in_data.sudt_amount != info_in_data.sudt_reserve {
        return Err(Error::SUDTReserveAmountDiff);
    }

    let req_lock_args: Vec<u8> = load_cell_lock(3, Source::Input)?.args().unpack();
    if LiquidityRequestLockArgs::from_raw(&req_lock_args).is_ok() {
        verify::liquidity_tx_verification()?;
    } else {
        verify::swap_tx_verification()?;
    }

    Ok(())
}

pub fn verify_info_creation(info_out_cell: &CellOutput) -> Result<(), Error> {
    // Todo: ignore verify type id temporary
    let _ = verify_type_id();

    let info_out_lock_args: Vec<u8> = info_out_cell.lock().args().unpack();
    let pool_type_hash = get_cell_type_hash!(1, Source::Output);
    let output_info_lock_count = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| {
            cell.lock().code_hash().unpack().as_ref() == hex::decode(INFO_LOCK_CODE_HASH).unwrap()
        })
        .count();

    if output_info_lock_count != 2 {
        return Err(Error::InfoCreationOutputCellCountMismatch);
    }

    if info_out_cell.lock().hash_type() != HashType::Data.into() {
        return Err(Error::InfoCellHashTypeMismatch);
    }

    let tmp = get_cell_type_hash!(0, Source::Output);
    debug!("{:?}", tmp);

    if info_out_lock_args[0..32] != blake2b!("ckb", pool_type_hash) {
        return Err(Error::InfoLockArgsFrontHalfMismatch);
    }

    if info_out_lock_args[32..64] != get_cell_type_hash!(0, Source::Output) {
        return Err(Error::InfoLockArgsSecondHalfMismatch);
    }

    if load_cell_lock_hash(0, Source::Output)? != load_cell_lock_hash(1, Source::Output)? {
        return Err(Error::InfoCreationCellLockHashMismatch);
    }

    if load_cell_data(1, Source::Output)?.len() < 16 {
        return Err(Error::CellDataLenTooShort);
    }

    Ok(())
}

#[allow(dead_code)]
enum HashType {
    Data,
    Code,
}

impl Into<Byte> for HashType {
    fn into(self) -> Byte {
        match self {
            HashType::Data => Byte::new(0u8),
            HashType::Code => Byte::new(1u8),
        }
    }
}
