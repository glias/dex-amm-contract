use alloc::vec::Vec;
use core::convert::TryInto;
use core::result::Result;

use num_bigint::BigUint;
use num_traits::identities::Zero;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use share::cell::{InfoCellData, LiquidityRequestLockArgs, SUDTAmountData};
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed::CellOutput, prelude::*},
    high_level::{load_cell, load_cell_data, load_cell_lock_hash, QueryIter},
};
use share::{decode_u128, get_cell_type_hash};

use crate::entry::{INFO_CAPACITY, INFO_VERSION, LIQUIDITY_CELL_BASE_INDEX, ONE, SUDT_CAPACITY};
use crate::error::Error;

pub fn liquidity_tx_verification(begin: usize, end: usize) -> Result<(), Error> {
    let info_in_data = InfoCellData::from_raw(&load_cell_data(0, Source::Input)?)?;
    let pool_in_cell = load_cell(1, Source::Input)?;
    let info_out_cell = load_cell(0, Source::Output)?;
    let info_out_data = InfoCellData::from_raw(&load_cell_data(0, Source::Output)?)?;
    let pool_out_cell = load_cell(1, Source::Output)?;
    let pool_out_data = SUDTAmountData::from_raw(&load_cell_data(1, Source::Output)?)?;
    let pool_type_hash = get_cell_type_hash!(1, Source::Input);

    let mut ckb_reserve = info_in_data.ckb_reserve;
    let mut sudt_reserve = info_in_data.sudt_reserve;
    let mut total_liquidity = info_in_data.total_liquidity;

    for (idx, (liquidity_order_cell, raw_data)) in QueryIter::new(load_cell, Source::Input)
        .zip(QueryIter::new(load_cell_data, Source::Input))
        .enumerate()
        .skip(3)
    {
        let raw_lock_args: Vec<u8> = liquidity_order_cell.lock().args().unpack();
        let liquidity_order_lock_args = LiquidityRequestLockArgs::from_raw(&raw_lock_args)?;
        if liquidity_order_lock_args.version != INFO_VERSION {
            return Err(Error::VersionDiff);
        }

        let liquidity_order_data = SUDTAmountData::from_raw(&raw_data)?;
        let liquidity_type_hash = get_cell_type_hash!(idx, Source::Input);
        if liquidity_order_lock_args.info_type_hash != get_cell_type_hash!(0, Source::Input) {
            return Err(Error::LiquidityArgsInfoTypeHashMismatch);
        }

        if info_in_data.total_liquidity == 0 {
            if QueryIter::new(load_cell, Source::Input).count() == 4 {
                verify_initial_mint(
                    info_in_data.liquidity_sudt_type_hash,
                    &mut ckb_reserve,
                    &mut sudt_reserve,
                    &mut total_liquidity,
                )?;
                break;
            } else {
                return Err(Error::InvalidInitialLiquidityTx);
            }
        }

        if liquidity_type_hash == info_in_data.liquidity_sudt_type_hash {
            burn_liquidity(
                idx,
                &liquidity_order_cell,
                liquidity_order_data.sudt_amount,
                &mut ckb_reserve,
                &mut sudt_reserve,
                &mut total_liquidity,
            )?;
        } else if liquidity_type_hash == pool_type_hash {
            mint_liquidity(
                idx,
                &info_in_data,
                &liquidity_order_cell,
                liquidity_order_data.sudt_amount,
                &mut ckb_reserve,
                &mut sudt_reserve,
                &mut total_liquidity,
            )?;
        } else {
            return Err(Error::UnknownLiquidity);
        }
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
        != (pool_in_cell.capacity().unpack() as u128 + info_out_data.ckb_reserve
            - info_in_data.ckb_reserve)
        || pool_out_data.sudt_amount != info_out_data.sudt_reserve
    {
        return Err(Error::InvalidCKBAmount);
    }

    Ok(())
}

fn verify_initial_mint(
    liquidity_sudt_type_hash: [u8; 32],
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
    total_liquidity: &mut u128,
) -> Result<(), Error> {
    if *ckb_reserve != 0 || *sudt_reserve != 0 || *total_liquidity != 0 {
        return Err(Error::InvalidInfoInData);
    }

    let order_cell = load_cell(3, Source::Input)?;
    let raw_lock_args: Vec<u8> = order_cell.lock().args().unpack();
    let order_lock_args = LiquidityRequestLockArgs::from_raw(&raw_lock_args)?;
    let order_data = SUDTAmountData::from_raw(&load_cell_data(3, Source::Input)?)?;
    let liquidity_sudt_data = SUDTAmountData::from_raw(&load_cell_data(3, Source::Output)?)?;

    if get_cell_type_hash!(3, Source::Output) != liquidity_sudt_type_hash {
        return Err(Error::LiquiditySUDTTypeHashMismatch);
    }

    if load_cell_lock_hash(3, Source::Output)?.as_ref() != order_lock_args.user_lock_hash.as_ref() {
        return Err(Error::LiquidityArgsUserLockHashMismatch);
    }

    let sudt_injected = order_data.sudt_amount;
    let ckb_injected = order_cell.capacity().unpack() - SUDT_CAPACITY;
    let user_liquidity = liquidity_sudt_data.sudt_amount;
    let mint_liquidity = (BigUint::from(sudt_injected) * ckb_injected).sqrt();

    if BigUint::from(user_liquidity) != mint_liquidity {
        return Err(Error::MintInitialLiquidityFailed);
    }

    *ckb_reserve += ckb_injected as u128;
    *sudt_reserve += sudt_injected;
    *total_liquidity += user_liquidity;

    Ok(())
}

fn mint_liquidity(
    liquidity_cell_index: usize,
    info_in_data: &InfoCellData,
    liquidity_order_cell: &CellOutput,
    liquidity_order_data: u128,
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
    total_liquidity: &mut u128,
) -> Result<(), Error> {
    if *total_liquidity == 0 {
        return Err(Error::UnknownLiquidity);
    }

    let cur_liquidity = *total_liquidity;
    let cur_ckb_reserve = *ckb_reserve;
    let cur_sudt_reserve = *sudt_reserve;

    let relative_index = liquidity_cell_index - LIQUIDITY_CELL_BASE_INDEX;
    let liquidity_index = relative_index * 2 + LIQUIDITY_CELL_BASE_INDEX;

    let raw_lock_args: Vec<u8> = liquidity_order_cell.lock().args().unpack();
    let liquidity_order_lock_args = LiquidityRequestLockArgs::from_raw(&raw_lock_args)?;
    let change_cell = load_cell(liquidity_index + 1, Source::Output)?;
    let change_lock_hash = load_cell_lock_hash(liquidity_index + 1, Source::Output)?;

    if get_cell_type_hash!(liquidity_index, Source::Output) != info_in_data.liquidity_sudt_type_hash
    {
        return Err(Error::LiquiditySUDTTypeHashMismatch);
    }

    if load_cell_lock_hash(liquidity_index, Source::Output)?
        != liquidity_order_lock_args.user_lock_hash
    {
        return Err(Error::LiquidityArgsUserLockHashMismatch);
    }

    let user_liquidity =
        SUDTAmountData::from_raw(&load_cell_data(liquidity_index, Source::Output)?)?.sudt_amount;

    let ckb_injected: u128;
    let sudt_injected: u128;
    let change_data = load_cell_data(liquidity_index + 1, Source::Output)?;

    if change_data.is_empty() {
        if change_cell.type_().is_some()
            || change_lock_hash != liquidity_order_lock_args.user_lock_hash
        {
            return Err(Error::InvalidChangeCell);
        }

        sudt_injected = liquidity_order_data;
        ckb_injected = liquidity_order_cell.capacity().unpack() as u128
            - SUDT_CAPACITY as u128
            - change_cell.capacity().unpack() as u128;

        if BigUint::from(ckb_injected)
            != (BigUint::from(sudt_injected) * cur_ckb_reserve / cur_sudt_reserve) + ONE
        {
            return Err(Error::LiquidityPoolTokenDiff);
        }

        let min_ckb_injected = liquidity_order_lock_args.amount_0 as u128;
        if min_ckb_injected == 0 || ckb_injected < min_ckb_injected {
            return Err(Error::InvalidMinCkbInject);
        }

        if BigUint::from(user_liquidity)
            != (BigUint::from(sudt_injected) * cur_liquidity / cur_sudt_reserve) + ONE
        {
            return Err(Error::SUDTInjectAmountDiff);
        }
    } else if change_data.len() >= 16 {
        if get_cell_type_hash!(liquidity_index + 1, Source::Output)
            != get_cell_type_hash!(1, Source::Input)
        {
            return Err(Error::SUDTTypeHashMismatch);
        }

        if change_lock_hash != liquidity_order_lock_args.user_lock_hash {
            return Err(Error::LiquidityArgsUserLockHashMismatch);
        }

        sudt_injected = liquidity_order_data - decode_u128(&change_data[0..16])?;
        ckb_injected = (liquidity_order_cell.capacity().unpack() - SUDT_CAPACITY * 2) as u128;

        if BigUint::from(sudt_injected)
            != (BigUint::from(ckb_injected) * cur_sudt_reserve / cur_ckb_reserve) + ONE
        {
            return Err(Error::LiquidityPoolTokenDiff);
        }

        let min_sudt_injected = liquidity_order_lock_args.amount_1;
        if min_sudt_injected == 0 || sudt_injected < min_sudt_injected {
            return Err(Error::InvalidMinSUDTInject);
        }

        if BigUint::from(user_liquidity)
            != (BigUint::from(ckb_injected) * cur_liquidity / cur_ckb_reserve) + ONE
        {
            return Err(Error::CKBInjectAmountDiff);
        }
    } else {
        return Err(Error::InvalidChangeCell);
    }

    *ckb_reserve += ckb_injected;
    *sudt_reserve += sudt_injected;
    *total_liquidity += user_liquidity;

    Ok(())
}

fn burn_liquidity(
    index: usize,
    liquidity_order_cell: &CellOutput,
    liquidity_order_data: u128,
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
    total_liquidity: &mut u128,
) -> Result<(), Error> {
    if *total_liquidity == 0 || liquidity_order_data == 0 {
        return Err(Error::BurnLiquidityFailed);
    }

    let cur_liquidity = *total_liquidity;
    let cur_ckb_reserve = *ckb_reserve;
    let cur_sudt_reserve = *sudt_reserve;

    let relative_index = index - LIQUIDITY_CELL_BASE_INDEX;
    let sudt_index = relative_index * 2 + LIQUIDITY_CELL_BASE_INDEX;

    let sudt_out = load_cell(sudt_index, Source::Output)?;
    let ckb_out = load_cell(sudt_index + 1, Source::Output)?;
    let sudt_data = load_cell_data(index, Source::Output)?;
    let raw_lock_args: Vec<u8> = liquidity_order_cell.lock().args().unpack();
    let liquidity_lock_args = LiquidityRequestLockArgs::from_raw(&raw_lock_args)?;

    if sudt_data.len() < 16 {
        return Err(Error::SUDTCellDataLenTooShort);
    }

    if !load_cell_data(sudt_index + 1, Source::Output)?.is_empty() {
        return Err(Error::CKBCellDataIsNotEmpty);
    }

    if get_cell_type_hash!(sudt_index, Source::Output) != get_cell_type_hash!(1, Source::Input) {
        return Err(Error::SUDTTypeHashMismatch);
    }

    if load_cell_lock_hash(sudt_index, Source::Output)? != liquidity_lock_args.user_lock_hash {
        return Err(Error::AddLiquiditySUDTOutLockHashMismatch);
    }

    if load_cell_lock_hash(sudt_index + 1, Source::Output)? != liquidity_lock_args.user_lock_hash {
        return Err(Error::AddLiquidityCkbOutLockHashMismatch);
    }

    let user_ckb_got = BigUint::from(sudt_out.capacity().unpack()) + ckb_out.capacity().unpack()
        - liquidity_order_cell.capacity().unpack();
    let user_sudt_got = BigUint::from(decode_u128(&sudt_data[0..16])?);
    let burned_liquidity = liquidity_order_data;

    let min_ckb_got = BigUint::from(liquidity_lock_args.amount_0);
    let min_sudt_got = BigUint::from(liquidity_lock_args.amount_1);
    let zero = BigUint::zero();

    if min_ckb_got == zero || user_ckb_got < min_ckb_got {
        return Err(Error::InvalidMinCkbGot);
    }

    if user_sudt_got < min_sudt_got {
        return Err(Error::InvalidMinSUDTGot);
    }

    if user_ckb_got != (BigUint::from(cur_ckb_reserve) * burned_liquidity / cur_liquidity) + ONE {
        return Err(Error::CKBGotAmountDiff);
    }

    if user_sudt_got != (BigUint::from(cur_sudt_reserve) * burned_liquidity / cur_liquidity) + ONE {
        return Err(Error::SUDTGotAmountDiff);
    }

    let user_ckb_got: u128 = user_ckb_got.try_into().unwrap();
    let user_sudt_got: u128 = user_sudt_got.try_into().unwrap();

    *ckb_reserve -= user_ckb_got;
    *sudt_reserve -= user_sudt_got;
    *total_liquidity -= burned_liquidity;

    Ok(())
}
