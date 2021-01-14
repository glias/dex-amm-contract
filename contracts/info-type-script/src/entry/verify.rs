use core::result::Result;

use num_bigint::{BigInt, BigUint};
use num_traits::identities::Zero;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use share::cell::{InfoCellData, LiquidityOrderLockArgs, SUDTAmountData};
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed::CellOutput, prelude::*},
    // debug,
    high_level::{load_cell, load_cell_data, load_cell_lock_hash, QueryIter},
};
use share::{decode_u128, get_cell_type_hash};

use crate::error::Error;

const INFO_VERSION: u8 = 1;

const THOUSAND: u128 = 1_000;
const TEN_THOUSAND: u128 = 10_000;
const SUDT_CAPACITY: u64 = 15_400_000_000;
const INFO_CAPACITY: u64 = 21_400_000_000;

pub fn liquidity_tx_verification() -> Result<(), Error> {
    let info_in_data = InfoCellData::from_raw(&load_cell_data(0, Source::Input)?)?;
    let pool_in_cell = load_cell(1, Source::Input)?;
    let info_out_cell = load_cell(0, Source::Output)?;
    let info_out_data = InfoCellData::from_raw(&load_cell_data(0, Source::Output)?)?;
    let pool_out_cell = load_cell(1, Source::Output)?;
    let pool_out_data = SUDTAmountData::from_raw(&load_cell_data(1, Source::Output)?)?;

    let mut liquidity_sudt_type_hash = [0u8; 20];
    liquidity_sudt_type_hash.copy_from_slice(&info_in_data.liquidity_sudt_type_hash);
    let pool_type_hash = get_cell_type_hash!(1, Source::Input);

    let ckb_reserve = info_in_data.ckb_reserve;
    let sudt_reserve = info_in_data.sudt_reserve;
    let total_liquidity = info_in_data.total_liquidity;

    let mut pool_ckb_paid = 0;
    let mut pool_sudt_paid = 0;
    let mut ckb_collect = 0;
    let mut sudt_collect = 0;
    let mut user_liquidity_mint = 0;
    let mut user_liquidity_burned = 0;
    let mut base_index = 0;

    for (idx, (liquidity_order_cell, raw_data)) in QueryIter::new(load_cell, Source::Input)
        .zip(QueryIter::new(load_cell_data, Source::Input))
        .enumerate()
        .skip(3)
    {
        let liquidity_order_lock_args =
            LiquidityOrderLockArgs::from_raw(&liquidity_order_cell.lock().args().as_slice())?;
        if liquidity_order_lock_args.version != INFO_VERSION {
            return Err(Error::VersionDiff);
        }

        // Todo: fix me
        let liquidity_order_data = SUDTAmountData::from_raw(&raw_data)?;
        let liquidity_type_hash = get_cell_type_hash!(idx, Source::Input);
        if liquidity_order_lock_args.info_type_hash.as_ref()
            != &get_cell_type_hash!(0, Source::Input)[0..20]
        {
            return Err(Error::InvalidLiquidityCell);
        }

        if info_in_data.total_liquidity == 0 {
            if QueryIter::new(load_cell, Source::Input).count() == 4 {
                return verify_initial_mint(
                    &info_in_data,
                    &mut ckb_collect,
                    &mut sudt_collect,
                    &mut user_liquidity_mint,
                );
            } else {
                return Err(Error::InvalidInitialLiquidityTx);
            }
        }

        if liquidity_type_hash[0..20] == info_in_data.liquidity_sudt_type_hash {
            burn_liquidity(
                idx,
                &liquidity_order_cell,
                liquidity_order_data.sudt_amount,
                ckb_reserve,
                sudt_reserve,
                total_liquidity,
                &mut pool_ckb_paid,
                &mut pool_sudt_paid,
                &mut user_liquidity_burned,
            )?;
            base_index = idx;
        } else if liquidity_type_hash[0..20] == pool_type_hash[0..20] {
            mint_liquidity(
                base_index,
                idx,
                &info_in_data,
                &liquidity_order_cell,
                liquidity_order_data.sudt_amount,
                ckb_reserve,
                sudt_reserve,
                total_liquidity,
                &mut ckb_collect,
                &mut sudt_collect,
                &mut user_liquidity_mint,
            )?;
        } else {
            return Err(Error::UnknownLiquidity);
        }
    }

    if info_out_cell.capacity().unpack() != INFO_CAPACITY
        || BigUint::from(info_out_data.ckb_reserve)
            != (BigUint::from(info_in_data.ckb_reserve) - pool_ckb_paid + ckb_collect)
        || BigUint::from(info_out_data.sudt_reserve)
            != (BigUint::from(info_in_data.sudt_reserve) - pool_sudt_paid + sudt_collect)
        || BigUint::from(info_out_data.total_liquidity)
            >= (BigUint::from(info_in_data.total_liquidity) * TEN_THOUSAND * 9995u128
                - BigUint::from(user_liquidity_burned)
                    * 9995u128
                    * 9995u128
                    * user_liquidity_burned
                + BigUint::from(user_liquidity_mint) * TEN_THOUSAND * TEN_THOUSAND)
    {
        return Err(Error::InvalidFee);
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

pub fn swap_tx_verification() -> Result<(), Error> {
    let info_in_data = InfoCellData::from_raw(&load_cell_data(0, Source::Input)?)?;
    let pool_in_cell = load_cell(1, Source::Input)?;
    let pool_in_data = SUDTAmountData::from_raw(&load_cell_data(1, Source::Input)?)?;
    let info_out_cell = load_cell(0, Source::Output)?;
    let info_out_data = InfoCellData::from_raw(&load_cell_data(0, Source::Output)?)?;
    let pool_out_cell = load_cell(1, Source::Output)?;
    let pool_out_data = SUDTAmountData::from_raw(&load_cell_data(1, Source::Output)?)?;

    if info_out_cell.capacity().unpack() != INFO_CAPACITY
        || info_out_data.total_liquidity != info_in_data.total_liquidity
    {
        return Err(Error::InvalidInfoData);
    }

    let ckb_got = BigInt::from(info_out_data.ckb_reserve) - info_in_data.ckb_reserve;
    let sudt_got = BigInt::from(info_out_data.sudt_reserve) - info_in_data.sudt_reserve;
    let ckb_reserve = info_in_data.ckb_reserve;
    let sudt_reserve = info_in_data.sudt_reserve;
    let zero = BigInt::zero();

    if ckb_got > zero && sudt_got < zero {
        let sudt_paid = info_in_data.sudt_reserve - info_out_data.sudt_reserve;
        if ckb_got.to_biguint().unwrap() * 998u128 * (sudt_reserve - sudt_paid)
            != BigUint::from(ckb_reserve) * sudt_paid * THOUSAND
        {
            return Err(Error::BuySUDTFailed);
        }
    } else if ckb_got < zero && sudt_got > zero {
        let ckb_paid = info_in_data.ckb_reserve - info_out_data.ckb_reserve;
        let tmp_sudt_got = sudt_got.to_biguint().unwrap();
        if tmp_sudt_got.clone() * 998u128 * ckb_reserve
            != BigUint::from(ckb_paid) * (sudt_reserve * THOUSAND + 998u128 * tmp_sudt_got)
        {
            return Err(Error::SellSUDTFailed);
        }
    } else {
        return Err(Error::InvalidInfoData);
    }

    if BigInt::from(pool_out_cell.capacity().unpack()) != pool_in_cell.capacity().unpack() + ckb_got
    {
        return Err(Error::CKBGotAmountDiff);
    } else if BigInt::from(pool_out_data.sudt_amount) != pool_in_data.sudt_amount + sudt_got {
        return Err(Error::SUDTGotAmountDiff);
    }

    Ok(())
}

fn verify_initial_mint(
    info_in_data: &InfoCellData,
    ckb_collect: &mut u128,
    sudt_collect: &mut u128,
    user_liquidity_mint: &mut u128,
) -> Result<(), Error> {
    if info_in_data.ckb_reserve != 0
        || info_in_data.sudt_reserve != 0
        || info_in_data.total_liquidity != 0
    {
        return Err(Error::InvalidInfoData);
    }

    let order_cell = load_cell(3, Source::Input)?;
    let order_lock_args = LiquidityOrderLockArgs::from_raw(order_cell.lock().args().as_slice())?;
    let order_data = SUDTAmountData::from_raw(&load_cell_data(3, Source::Input)?)?;
    let liquidity_sudt_data = SUDTAmountData::from_raw(&load_cell_data(3, Source::Output)?)?;

    if get_cell_type_hash!(3, Source::Output)[0..20] != info_in_data.liquidity_sudt_type_hash
        || load_cell_lock_hash(3, Source::Output)?.as_ref()
            != order_lock_args.user_lock_hash.as_ref()
    {
        return Err(Error::InvalidLiquidityCell);
    }

    let sudt_injected = order_data.sudt_amount;
    let ckb_injected = order_cell.capacity().unpack() - SUDT_CAPACITY;
    let user_liquidity = liquidity_sudt_data.sudt_amount;
    let mint_liquidity = (BigUint::from(sudt_injected) * ckb_injected).sqrt();

    if BigUint::from(user_liquidity) * THOUSAND != mint_liquidity * 9995u128 {
        return Err(Error::MintInitialLiquidityFailed);
    }

    *ckb_collect += ckb_injected as u128;
    *sudt_collect += sudt_injected;
    *user_liquidity_mint += user_liquidity;
    Ok(())
}

fn mint_liquidity(
    base_index: usize,
    liquidity_cell_index: usize,
    info_in_data: &InfoCellData,
    liquidity_order_cell: &CellOutput,
    liquidity_order_data: u128,
    ckb_reserve: u128,
    sudt_reserve: u128,
    total_liquidity: u128,
    ckb_collect: &mut u128,
    sudt_collect: &mut u128,
    user_liquidity_mint: &mut u128,
) -> Result<(), Error> {
    if total_liquidity == 0 {
        return Err(Error::UnknownLiquidity);
    }

    let relative_index = liquidity_cell_index - base_index;
    let liquidity_index = relative_index * 2 + base_index;
    let liquidity_cell = load_cell(liquidity_index, Source::Output)?;
    let liquidity_order_lock_args =
        LiquidityOrderLockArgs::from_raw(liquidity_order_cell.lock().args().as_slice())?;
    let change_cell = load_cell(liquidity_index + 1, Source::Output)?;
    let change_lock_hash = load_cell_lock_hash(liquidity_index + 1, Source::Output)?;

    if get_cell_type_hash!(liquidity_index, Source::Output)[0..20]
        != info_in_data.liquidity_sudt_type_hash
        || load_cell_lock_hash(liquidity_index, Source::Output)?.as_ref()
            != liquidity_order_lock_args.user_lock_hash.as_ref()
    {
        return Err(Error::InvalidLiquidityCell);
    }

    let user_liquidity =
        SUDTAmountData::from_raw(&load_cell_data(liquidity_index, Source::Output)?)?.sudt_amount;

    let ckb_injected: u128;
    let sudt_injected: u128;
    let change_data = load_cell_data(liquidity_index + 1, Source::Output)?;

    if change_data.is_empty() {
        if change_cell.type_().is_none()
            || change_lock_hash.as_ref() != liquidity_order_lock_args.user_lock_hash.as_ref()
        {
            return Err(Error::InvalidChangeCell);
        }

        sudt_injected = liquidity_order_data;
        ckb_injected = liquidity_cell.capacity().unpack() as u128
            - SUDT_CAPACITY as u128
            - change_cell.capacity().unpack() as u128;

        if BigUint::from(sudt_reserve) * ckb_injected
            != BigUint::from(sudt_injected) * ckb_reserve + sudt_reserve
        {
            return Err(Error::LiquidityPoolTokenDiff);
        }

        let min_ckb_injected = liquidity_order_lock_args.amount_0;
        if min_ckb_injected == 0 || ckb_injected < min_ckb_injected {
            return Err(Error::InvalidLiquidityCell);
        }

        if BigUint::from(user_liquidity) * TEN_THOUSAND * sudt_reserve
            != BigUint::from(sudt_injected) * 9995u128 * total_liquidity
        {
            return Err(Error::LiquidityPoolTokenDiff);
        }
    } else if change_data.len() >= 16 {
        if get_cell_type_hash!(liquidity_index + 1, Source::Output)
            != get_cell_type_hash!(1, Source::Input)
            || change_lock_hash.as_ref() != liquidity_order_lock_args.user_lock_hash.as_ref()
        {
            return Err(Error::InvalidChangeCell);
        }

        sudt_injected = liquidity_order_data - decode_u128(&change_data[0..16])?;
        ckb_injected = (liquidity_cell.capacity().unpack() - SUDT_CAPACITY * 2) as u128;

        if BigUint::from(ckb_reserve) * sudt_injected
            != BigUint::from(ckb_injected) * sudt_reserve + ckb_reserve
        {
            return Err(Error::LiquidityPoolTokenDiff);
        }

        let min_sudt_injected = liquidity_order_lock_args.amount_1;
        if min_sudt_injected == 0 || sudt_injected < min_sudt_injected {
            return Err(Error::InvalidLiquidityCell);
        }

        if BigUint::from(user_liquidity) * TEN_THOUSAND * ckb_reserve
            != BigUint::from(ckb_injected) * 9995u128 * total_liquidity
        {
            return Err(Error::LiquidityPoolTokenDiff);
        }
    } else {
        return Err(Error::InvalidChangeCell);
    }

    *ckb_collect += ckb_injected;
    *sudt_collect += sudt_injected;
    *user_liquidity_mint += user_liquidity;

    Ok(())
}

fn burn_liquidity(
    index: usize,
    liquidity_order_cell: &CellOutput,
    liquidity_order_data: u128,
    ckb_reserve: u128,
    sudt_reserve: u128,
    total_liquidity: u128,
    pool_ckb_paid: &mut u128,
    pool_sudt_paid: &mut u128,
    user_liquidity_burned: &mut u128,
) -> Result<(), Error> {
    if total_liquidity == 0 || liquidity_order_data == 0 {
        return Err(Error::BurnLiquidityFailed);
    }

    let ckb_sudt_out = load_cell(index, Source::Output)?;
    let ckb_sudt_data = load_cell_data(index, Source::Output)?;
    let liquidity_lock_args =
        LiquidityOrderLockArgs::from_raw(liquidity_order_cell.lock().args().as_slice())?;

    if ckb_sudt_data.len() < 16
        || get_cell_type_hash!(index, Source::Output) != get_cell_type_hash!(1, Source::Input)
        || load_cell_lock_hash(index, Source::Output)?.as_ref()
            != liquidity_lock_args.user_lock_hash.as_ref()
    {
        return Err(Error::InvalidLiquidityCell);
    }

    let user_ckb_got =
        (ckb_sudt_out.capacity().unpack() - liquidity_order_cell.capacity().unpack()) as u128;
    let user_sudt_got = decode_u128(&ckb_sudt_data[0..16])?;
    let burned_liquidity = liquidity_order_data;
    let min_ckb_got = liquidity_lock_args.amount_0;
    let min_sudt_got = liquidity_lock_args.amount_1;

    if min_ckb_got == 0 || user_ckb_got < min_ckb_got || user_sudt_got < min_sudt_got {
        return Err(Error::InvalidLiquidityCell);
    }

    if BigUint::from(user_ckb_got) * TEN_THOUSAND * total_liquidity
        != BigUint::from(ckb_reserve) * 9995u128 * burned_liquidity
        || BigUint::from(user_sudt_got) * TEN_THOUSAND * total_liquidity
            != BigUint::from(sudt_reserve) * 9995u128 * burned_liquidity
    {
        return Err(Error::LiquidityPoolTokenDiff);
    }

    *pool_ckb_paid += user_ckb_got;
    *pool_sudt_paid += user_sudt_got;
    *user_liquidity_burned += burned_liquidity;

    debug_assert!(*pool_ckb_paid < ckb_reserve);
    debug_assert!(*pool_sudt_paid < sudt_reserve);
    Ok(())
}
