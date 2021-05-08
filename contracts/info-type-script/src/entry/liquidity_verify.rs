use alloc::vec::Vec;
use core::convert::TryInto;
use core::result::Result;

use num_bigint::BigUint;
use num_traits::identities::Zero;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed::CellOutput, prelude::*},
    high_level::{load_cell, load_cell_data, load_cell_lock_hash, QueryIter},
};
use share::{cell::LiquidityRequestLockArgs, decode_u128, get_cell_type_hash};

use crate::entry::{INFO_INDEX, INFO_VERSION, ONE, POOL_INDEX, SUDT_CAPACITY, SUDT_CELL_DATA_LEN};
use crate::error::Error;

pub fn liquidity_tx_verification(
    swap_cell_count: usize,
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
    total_liquidity: &mut u128,
    liquidity_sudt_type_hash: [u8; 32],
) -> Result<(), Error> {
    let info_in_type_hash = get_cell_type_hash!(INFO_INDEX, Source::Input);
    let pool_type_hash = get_cell_type_hash!(POOL_INDEX, Source::Input);
    let input_cell_count = QueryIter::new(load_cell, Source::Input).count();
    let base_index = swap_cell_count + 3;

    for idx in base_index..input_cell_count {
        let liquidity_order_cell = load_cell(idx, Source::Input)?;
        let raw_lock_args: Vec<u8> = liquidity_order_cell.lock().args().unpack();
        let liquidity_order_lock_args = LiquidityRequestLockArgs::from_raw(&raw_lock_args)?;

        if liquidity_order_lock_args.version != INFO_VERSION {
            return Err(Error::VersionDiff);
        }

        let liquidity_order_data = decode_u128(&load_cell_data(idx, Source::Input)?)?;
        let liquidity_type_hash = get_cell_type_hash!(idx, Source::Input);
        if liquidity_order_lock_args.info_type_hash != info_in_type_hash {
            return Err(Error::LiquidityArgsInfoTypeHashMismatch);
        }

        if liquidity_type_hash == liquidity_sudt_type_hash {
            burn_liquidity(
                idx,
                base_index,
                &liquidity_order_cell,
                liquidity_order_data,
                ckb_reserve,
                sudt_reserve,
                total_liquidity,
            )?;
        } else if liquidity_type_hash == pool_type_hash {
            mint_liquidity(
                idx,
                base_index,
                pool_type_hash,
                liquidity_sudt_type_hash,
                &liquidity_order_cell,
                liquidity_order_data,
                ckb_reserve,
                sudt_reserve,
                total_liquidity,
            )?;
        } else {
            return Err(Error::UnknownLiquidity);
        }
    }

    Ok(())
}

pub fn verify_initial_mint(
    liquidity_sudt_type_hash: [u8; 32],
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
    total_liquidity: &mut u128,
) -> Result<(), Error> {
    if *ckb_reserve != 0 || *sudt_reserve != 0 || *total_liquidity != 0 {
        return Err(Error::InvalidInfoInData);
    }

    let order_cell_index: usize = 3;
    let order_cell = load_cell(order_cell_index, Source::Input)?;
    let raw_lock_args: Vec<u8> = order_cell.lock().args().unpack();
    let order_lock_args = LiquidityRequestLockArgs::from_raw(&raw_lock_args)?;
    let order_data = decode_u128(&load_cell_data(order_cell_index, Source::Input)?)?;
    let liquidity_sudt_data = decode_u128(&load_cell_data(order_cell_index, Source::Output)?)?;

    if get_cell_type_hash!(order_cell_index, Source::Output) != liquidity_sudt_type_hash {
        return Err(Error::LiquiditySUDTTypeHashMismatch);
    }

    if load_cell_lock_hash(order_cell_index, Source::Output)?.as_ref()
        != order_lock_args.user_lock_hash.as_ref()
    {
        return Err(Error::LiquidityArgsUserLockHashMismatch);
    }

    let sudt_injected = order_data;
    let ckb_injected = order_cell.capacity().unpack() - SUDT_CAPACITY;
    let user_liquidity = liquidity_sudt_data;
    let mint_liquidity = (BigUint::from(sudt_injected) * ckb_injected).sqrt();

    if user_liquidity == 0 {
        return Err(Error::MintLiquidityEqZero);
    }

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
    base_index: usize,
    pool_type_hash: [u8; 32],
    liquidity_sudt_type_hash: [u8; 32],
    liquidity_order_cell: &CellOutput,
    liquidity_order_data: u128,
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
    total_liquidity: &mut u128,
) -> Result<(), Error> {
    if *total_liquidity == 0 {
        return Err(Error::UnknownLiquidity);
    }

    let relative_index = liquidity_cell_index - base_index;
    let liquidity_index = relative_index * 2 + base_index;

    let raw_lock_args: Vec<u8> = liquidity_order_cell.lock().args().unpack();
    let liquidity_order_lock_args = LiquidityRequestLockArgs::from_raw(&raw_lock_args)?;
    let change_cell = load_cell(liquidity_index + 1, Source::Output)?;
    let change_lock_hash = load_cell_lock_hash(liquidity_index + 1, Source::Output)?;

    if get_cell_type_hash!(liquidity_index, Source::Output) != liquidity_sudt_type_hash {
        return Err(Error::LiquiditySUDTTypeHashMismatch);
    }

    if load_cell_lock_hash(liquidity_index, Source::Output)?
        != liquidity_order_lock_args.user_lock_hash
    {
        return Err(Error::LiquidityArgsUserLockHashMismatch);
    }

    let user_liquidity = decode_u128(&load_cell_data(liquidity_index, Source::Output)?)?;
    let change_data = load_cell_data(liquidity_index + 1, Source::Output)?;

    let ckb_injected: u128;
    let sudt_injected: u128;

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

        if ckb_injected == 0 || sudt_injected == 0 {
            return Err(Error::InjectAmountEqZero);
        }

        if BigUint::from(ckb_injected)
            != (BigUint::from(sudt_injected) * (*ckb_reserve) / *sudt_reserve) + ONE
        {
            return Err(Error::LiquidityPoolTokenDiff);
        }

        let min_ckb_injected = liquidity_order_lock_args.amount_0 as u128;
        if min_ckb_injected == 0 || ckb_injected < min_ckb_injected {
            return Err(Error::InvalidMinCkbInject);
        }

        if BigUint::from(user_liquidity)
            != (BigUint::from(sudt_injected) * (*total_liquidity) / *sudt_reserve) + ONE
        {
            return Err(Error::SUDTInjectAmountDiff);
        }
    } else if change_data.len() >= SUDT_CELL_DATA_LEN {
        if get_cell_type_hash!(liquidity_index + 1, Source::Output) != pool_type_hash {
            return Err(Error::SUDTTypeHashMismatch);
        }

        if change_lock_hash != liquidity_order_lock_args.user_lock_hash {
            return Err(Error::LiquidityArgsUserLockHashMismatch);
        }

        sudt_injected = liquidity_order_data - decode_u128(&change_data[0..16])?;
        ckb_injected = (liquidity_order_cell.capacity().unpack() - SUDT_CAPACITY * 2) as u128;

        if ckb_injected == 0 || sudt_injected == 0 {
            return Err(Error::InjectAmountEqZero);
        }

        if BigUint::from(sudt_injected)
            != (BigUint::from(ckb_injected) * (*sudt_reserve) / (*ckb_reserve)) + ONE
        {
            return Err(Error::LiquidityPoolTokenDiff);
        }

        let min_sudt_injected = liquidity_order_lock_args.amount_1;
        if min_sudt_injected == 0 || sudt_injected < min_sudt_injected {
            return Err(Error::InvalidMinSUDTInject);
        }

        if BigUint::from(user_liquidity)
            != (BigUint::from(ckb_injected) * (*total_liquidity) / (*ckb_reserve)) + ONE
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
    base_index: usize,
    liquidity_order_cell: &CellOutput,
    liquidity_order_data: u128,
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
    total_liquidity: &mut u128,
) -> Result<(), Error> {
    if *total_liquidity == 0 || liquidity_order_data == 0 {
        return Err(Error::BurnLiquidityFailed);
    }

    let relative_index = index - base_index;
    let sudt_index = relative_index * 2 + base_index;

    let sudt_out = load_cell(sudt_index, Source::Output)?;
    let ckb_out = load_cell(sudt_index + 1, Source::Output)?;
    let sudt_data = load_cell_data(sudt_index, Source::Output)?;
    let raw_lock_args: Vec<u8> = liquidity_order_cell.lock().args().unpack();
    let liquidity_lock_args = LiquidityRequestLockArgs::from_raw(&raw_lock_args)?;

    if sudt_data.len() < SUDT_CELL_DATA_LEN {
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

    if user_ckb_got < min_ckb_got {
        return Err(Error::InvalidMinCkbGot);
    }

    if user_sudt_got < min_sudt_got {
        return Err(Error::InvalidMinSUDTGot);
    }

    if user_ckb_got != (BigUint::from(*ckb_reserve) * burned_liquidity / *total_liquidity) + ONE {
        return Err(Error::CKBGotAmountDiff);
    }

    if user_sudt_got != (BigUint::from(*sudt_reserve) * burned_liquidity / *total_liquidity) + ONE {
        return Err(Error::SUDTGotAmountDiff);
    }

    let user_ckb_got: u128 = user_ckb_got.try_into().unwrap();
    let user_sudt_got: u128 = user_sudt_got.try_into().unwrap();

    *ckb_reserve -= user_ckb_got;
    *sudt_reserve -= user_sudt_got;
    *total_liquidity -= burned_liquidity;

    Ok(())
}
