use num_bigint::{BigInt, BigUint};
use num_traits::identities::Zero;

use share::cell::{InfoCellData, SUDTAmountData};
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    high_level::{load_cell, load_cell_data},
};

use crate::entry::{FEE_RATE, INFO_CAPACITY, ONE, THOUSAND};
use crate::error::Error;

pub fn swap_tx_verification() -> Result<(), Error> {
    let info_in_data = InfoCellData::from_raw(&load_cell_data(0, Source::Input)?)?;
    let pool_in_cell = load_cell(1, Source::Input)?;
    let pool_in_data = SUDTAmountData::from_raw(&load_cell_data(1, Source::Input)?)?;
    let info_out_cell = load_cell(0, Source::Output)?;
    let info_out_data = InfoCellData::from_raw(&load_cell_data(0, Source::Output)?)?;
    let pool_out_cell = load_cell(1, Source::Output)?;
    let pool_out_data = SUDTAmountData::from_raw(&load_cell_data(1, Source::Output)?)?;

    if info_out_cell.capacity().unpack() != INFO_CAPACITY {
        return Err(Error::InfoCapacityDiff);
    }

    if info_out_data.total_liquidity != info_in_data.total_liquidity {
        return Err(Error::InAndOutLiquidityDiff);
    }

    let ckb_got = BigInt::from(info_out_data.ckb_reserve) - info_in_data.ckb_reserve;
    let sudt_got = BigInt::from(info_out_data.sudt_reserve) - info_in_data.sudt_reserve;
    let ckb_reserve = info_in_data.ckb_reserve;
    let sudt_reserve = info_in_data.sudt_reserve;
    let zero = BigInt::zero();

    if ckb_got > zero && sudt_got < zero {
        // Ckb -> SUDT
        let sudt_paid = info_in_data.sudt_reserve - info_out_data.sudt_reserve;
        let tmp_ckb_got = ckb_got.to_biguint().unwrap();
        let numerator = tmp_ckb_got.clone() * FEE_RATE * sudt_reserve;
        let denominator = ckb_reserve * THOUSAND + tmp_ckb_got * FEE_RATE;

        if BigUint::from(sudt_paid) != numerator / denominator + ONE {
            return Err(Error::BuySUDTFailed);
        }
    } else if ckb_got < zero && sudt_got > zero {
        // SUDT -> Ckb
        let ckb_paid = info_in_data.ckb_reserve - info_out_data.ckb_reserve;
        let tmp_sudt_got = sudt_got.to_biguint().unwrap();
        let numerator = tmp_sudt_got.clone() * FEE_RATE * ckb_reserve;
        let denominator = sudt_reserve * THOUSAND + FEE_RATE * tmp_sudt_got;

        if BigUint::from(ckb_paid) != numerator / denominator + ONE {
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
