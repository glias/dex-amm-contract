use alloc::vec::Vec;

use num_bigint::BigUint;
use share::cell::{InfoCellData, SUDTAmountData, SwapRequestLockArgs};
use share::ckb_std::ckb_types::packed::CellOutput;
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    high_level::{load_cell, load_cell_data, load_cell_lock_hash, QueryIter},
};
use share::{decode_u128, get_cell_type_hash};

use crate::entry::{FEE_RATE, INFO_CAPACITY, ONE, SUDT_CAPACITY, THOUSAND};
use crate::error::Error;

pub fn swap_tx_verification(
    info_out_cell: &CellOutput,
    info_out_data: &InfoCellData,
    pool_in_cell: &CellOutput,
    pool_in_data: &SUDTAmountData,
    pool_out_cell: &CellOutput,
    pool_out_data: &SUDTAmountData,
    swap_cell_count: usize,
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
    total_liquidity: &mut u128,
) -> Result<(), Error> {
    if info_out_cell.capacity().unpack() != INFO_CAPACITY {
        return Err(Error::InfoCapacityDiff);
    }

    for (idx, req_cell) in QueryIter::new(load_cell, Source::Input).enumerate().skip(3) {
        let raw_lock_args: Vec<u8> = req_cell.lock().args().unpack();
        let req_lock_args = SwapRequestLockArgs::from_raw(&raw_lock_args)?;
        let output_cell = load_cell(idx, Source::Output)?;

        if load_cell_lock_hash(idx, Source::Output)? != req_lock_args.user_lock_hash {
            return Err(Error::InvalidOutputLockHash);
        }

        if req_cell.type_().is_none() {
            verify_ckb_exchange_sudt(
                idx,
                &req_cell,
                &req_lock_args,
                &output_cell,
                &mut ckb_reserve,
                &mut sudt_reserve,
            )?;
        } else {
            verify_sudt_exchange_ckb(
                idx,
                &req_cell,
                &req_lock_args,
                &output_cell,
                &mut ckb_reserve,
                &mut sudt_reserve,
            )?;
        }
    }

    Ok(())
}

fn verify_ckb_exchange_sudt(
    index: usize,
    req_cell: &CellOutput,
    req_lock_args: &SwapRequestLockArgs,
    output_cell: &CellOutput,
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
) -> Result<(), Error> {
    let req_capcity = req_cell.capacity().unpack();
    let output_capcity = output_cell.capacity().unpack();
    let ckb_paid = req_capcity - SUDT_CAPACITY;

    if ckb_paid == 0 {
        return Err(Error::RequestCapcityEqSUDTCapcity);
    }

    if req_lock_args.sudt_type_hash != get_cell_type_hash!(index, Source::Output) {
        return Err(Error::InvalidOutputTypeHash);
    }

    if req_capcity <= output_capcity || req_capcity - output_capcity != ckb_paid {
        return Err(Error::InvalidSwapOutputCapacity);
    }

    let sudt_got = decode_u128(&load_cell_data(index, Source::Output)?)?;
    if sudt_got < req_lock_args.min_amount_out {
        return Err(Error::SwapAmountLessThanMin);
    }

    let numerator = BigUint::from(sudt_got) * FEE_RATE * (*ckb_reserve);
    let denominator = (*sudt_reserve) * THOUSAND + FEE_RATE * BigUint::from(sudt_got);

    if BigUint::from(ckb_paid) != numerator / denominator + ONE {
        return Err(Error::SellSUDTFailed);
    }

    *ckb_reserve -= ckb_paid as u128;
    *sudt_reserve += sudt_got;

    Ok(())
}

fn verify_sudt_exchange_ckb(
    index: usize,
    req_cell: &CellOutput,
    req_lock_args: &SwapRequestLockArgs,
    output_cell: &CellOutput,
    ckb_reserve: &mut u128,
    sudt_reserve: &mut u128,
) -> Result<(), Error> {
    let sudt_paid = decode_u128(&load_cell_data(index, Source::Input)?)?;

    if sudt_paid == 0 {
        return Err(Error::SwapInputSUDTAmountEqZero);
    }

    if output_cell.type_().is_some() {
        return Err(Error::InvalidOutputTypeHash);
    }

    let ckb_got = (output_cell.capacity().unpack() - req_cell.capacity().unpack()) as u128;
    if ckb_got < req_lock_args.min_amount_out {
        return Err(Error::InvalidSwapOutputCapacity);
    }

    if !load_cell_data(index, Source::Output)?.is_empty() {
        return Err(Error::InvalidSwapOutputData);
    }

    let numerator = BigUint::from(ckb_got) * FEE_RATE * (*sudt_reserve);
    let denominator = (*ckb_reserve) * THOUSAND + BigUint::from(ckb_got) * FEE_RATE;

    if BigUint::from(sudt_paid) != numerator / denominator + ONE {
        return Err(Error::BuySUDTFailed);
    }

    *ckb_reserve += ckb_got;
    *sudt_reserve -= sudt_paid;

    Ok(())
}
