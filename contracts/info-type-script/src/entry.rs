mod liquidity_verify;
mod swap_verify;
mod type_id;

use alloc::vec::Vec;
use core::result::Result;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use share::cell::{InfoCellData, SUDTAmountData};
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        packed::{Byte, CellOutput},
        prelude::*,
    },
    default_alloc,
    high_level::{
        load_cell, load_cell_data, load_cell_lock_hash, load_cell_type_hash, load_script,
        load_witness_args, QueryIter,
    },
};
use share::{blake2b, decode_u64, get_cell_type_hash, hash::blake2b_256};

use crate::error::Error;

const ONE: u128 = 1;
const THOUSAND: u128 = 1_000;
const FEE_RATE: u128 = 997;
const POOL_CAPACITY: u128 = 18_600_000_000;
const SUDT_CAPACITY: u64 = 14_200_000_000;
const INFO_CAPACITY: u64 = 25_000_000_000;
const INFO_VERSION: u8 = 1;

pub static INFO_LOCK_CODE_HASH: &str =
    include!(concat!(env!("OUT_DIR"), "/info_lock_code_hash.rs"));

// Alloc 4K fast HEAP + 2M HEAP to receives PrefilledData
default_alloc!(4 * 1024, 2048 * 1024, 64);

pub fn main() -> Result<(), Error> {
    let info_type_code_hash = load_script()?.code_hash().unpack();
    let (input_info_cell_count, output_info_cell_count) = get_info_count(info_type_code_hash);

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
    let info_out_cell = load_cell(0, Source::Output)?;
    let info_out_data = InfoCellData::from_raw(&load_cell_data(0, Source::Output)?)?;
    let pool_out_cell = load_cell(1, Source::Output)?;
    let pool_out_data = SUDTAmountData::from_raw(&load_cell_data(1, Source::Output)?)?;

    let mut ckb_reserve = info_in_data.ckb_reserve;
    let mut sudt_reserve = info_in_data.sudt_reserve;
    let mut total_liquidity = info_in_data.total_liquidity;
    let liquidity_sudt_type_hash = info_in_data.liquidity_sudt_type_hash;

    basic_verify(&info_in_data, &pool_in_cell, &pool_in_data)?;

    let raw_witness: Vec<u8> = load_witness_args(0, Source::Input)?
        .input_type()
        .to_opt()
        .unwrap()
        .unpack();
    let swap_cell_count = decode_u64(&raw_witness)? as usize;
    let output_cell_count = QueryIter::new(load_cell, Source::Output).count();

    if output_cell_count == 4 && swap_cell_count == 0 {
        liquidity_verify::verify_initial_mint(
            liquidity_sudt_type_hash,
            &mut ckb_reserve,
            &mut sudt_reserve,
            &mut total_liquidity,
        )?;
    } else {
        swap_verify::swap_tx_verification(
            &info_out_cell,
            swap_cell_count,
            &mut ckb_reserve,
            &mut sudt_reserve,
        )?;

        liquidity_verify::liquidity_tx_verification(
            swap_cell_count,
            &mut ckb_reserve,
            &mut sudt_reserve,
            &mut total_liquidity,
            liquidity_sudt_type_hash,
        )?;
    }

    if info_out_cell.capacity().unpack() != INFO_CAPACITY
        || info_out_data.ckb_reserve != ckb_reserve
    {
        return Err(Error::InvalidCKBReserve);
    }

    if info_out_data.sudt_reserve != sudt_reserve {
        return Err(Error::InvalidSUDTReserve);
    }

    if info_out_data.total_liquidity != total_liquidity {
        return Err(Error::InvalidTotalLiquidity);
    }

    if (pool_out_cell.capacity().unpack() as u128)
        != (pool_in_cell.capacity().unpack() as u128 + ckb_reserve - info_in_data.ckb_reserve)
    {
        return Err(Error::InvalidOutputPoolCapacity);
    }

    if pool_out_data.sudt_amount
        != pool_in_data.sudt_amount + sudt_reserve - info_in_data.sudt_reserve
        || pool_out_data.sudt_amount != info_out_data.sudt_reserve
    {
        return Err(Error::InvalidPoolOutputData);
    }

    Ok(())
}

fn get_info_count(info_type_code_hash: [u8; 32]) -> (usize, usize) {
    let input_count = QueryIter::new(load_cell, Source::Input)
        .filter(|cell| {
            cell.type_().to_opt().map_or_else(
                || false,
                |script| script.code_hash().unpack() == info_type_code_hash,
            )
        })
        .count();
    let output_count = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| {
            cell.type_().to_opt().map_or_else(
                || false,
                |script| script.code_hash().unpack() == info_type_code_hash,
            )
        })
        .count();

    (input_count, output_count)
}

fn basic_verify(
    info_in_data: &InfoCellData,
    pool_in_cell: &CellOutput,
    pool_in_data: &SUDTAmountData,
) -> Result<(), Error> {
    if (pool_in_cell.capacity().unpack() as u128) != POOL_CAPACITY + info_in_data.ckb_reserve {
        return Err(Error::CKBReserveAmountDiff);
    }

    if pool_in_data.sudt_amount != info_in_data.sudt_reserve {
        return Err(Error::SUDTReserveAmountDiff);
    }

    Ok(())
}

fn verify_info_creation(info_out_cell: &CellOutput) -> Result<(), Error> {
    type_id::verify_type_id()?;

    let info_out_lock_args: Vec<u8> = info_out_cell.lock().args().unpack();
    let pool_type_hash = get_cell_type_hash!(1, Source::Output);
    let (output_info_lock_count, is_data_deploy) = get_info_cell_count()?;

    if output_info_lock_count != 2 {
        if is_data_deploy {
            return Err(Error::InvalidInfoLockCountInOutput);
        } else {
            return Err(Error::InvalidInfoLockCountInCellDeps);
        }
    }

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

fn get_info_cell_count() -> Result<(usize, bool), Error> {
    let info_lock_data_hash = hex::decode(INFO_LOCK_CODE_HASH).unwrap();

    let ret = if load_cell(0, Source::Output)?.lock().hash_type() == HashType::Code.as_byte() {
        let is_data_deploy = false;
        (type_deploy(&info_lock_data_hash)?, is_data_deploy)
    } else {
        let count = QueryIter::new(load_cell, Source::Output)
            .filter(|cell| cell.lock().code_hash().unpack() == info_lock_data_hash.as_ref())
            .count();
        (count, true)
    };

    Ok(ret)
}

fn type_deploy(info_lock_data_hash: &[u8]) -> Result<usize, Error> {
    let mut flag = false;
    let info_lock_code_hash = load_cell(0, Source::Output)?.lock().code_hash().unpack();

    for (idx, res) in QueryIter::new(load_cell_type_hash, Source::CellDep).enumerate() {
        if let Some(hash) = res {
            if hash == info_lock_code_hash
                && blake2b_256(load_cell_data(idx, Source::CellDep)?) == info_lock_data_hash
            {
                flag = true;
                break;
            }
        }
    }

    if flag {
        let ret = QueryIter::new(load_cell, Source::Output)
            .filter(|cell| cell.lock().code_hash().unpack() == info_lock_code_hash)
            .count();
        Ok(ret)
    } else {
        Err(Error::NoInfoLockInCellDeps)
    }
}

#[allow(dead_code)]
enum HashType {
    Data,
    Code,
}

impl HashType {
    fn as_byte(&self) -> Byte {
        match self {
            HashType::Data => Byte::new(0u8),
            HashType::Code => Byte::new(1u8),
        }
    }
}
