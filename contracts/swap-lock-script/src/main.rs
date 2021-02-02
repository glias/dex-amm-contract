//! Generated by capsule
//!
//! `main.rs` is used to define rust lang items and modules.
//! See `entry.rs` for the `main` function.
//! See `error.rs` for the `Error` type.

#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

mod error;

use alloc::vec::Vec;
use core::result::Result;

use num_bigint::BigUint;
use share::cell::SwapRequestLockArgs;
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    default_alloc,
    high_level::{
        load_cell, load_cell_data, load_cell_lock_hash, load_script, load_script_hash,
        load_witness_args, QueryIter,
    },
};
use share::{ckb_std, decode_u128, get_cell_type_hash};

use crate::error::Error;

const SUDT_CAPACITY: u64 = 15_400_000_000;

// Alloc 4K fast HEAP + 2M HEAP to receives PrefilledData
default_alloc!(4 * 1024, 2048 * 1024, 64);

ckb_std::entry!(program_entry);

/// program entry
fn program_entry() -> i8 {
    // Call main function and return error code
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

fn main() -> Result<(), Error> {
    let script_args: Vec<u8> = load_script()?.args().unpack();

    // Cancel request
    for (idx, lock_hash) in QueryIter::new(load_cell_lock_hash, Source::Input).enumerate() {
        if lock_hash == script_args[49..81]
            && load_witness_args(idx, Source::Input)?.total_size() != 0
        {
            return Ok(());
        }
    }

    let self_hash = load_script_hash()?;
    let group_input_index = QueryIter::new(load_cell_lock_hash, Source::Input)
        .enumerate()
        .filter_map(|(idx, hash)| if hash == self_hash { Some(idx) } else { None })
        .collect::<Vec<_>>();
    let req_lock_args = SwapRequestLockArgs::from_raw(&script_args)?;

    for index in group_input_index {
        let req_cell = load_cell(index, Source::Input)?;
        let output_cell = load_cell(index, Source::Output)?;

        if load_cell_lock_hash(index, Source::Output)? != req_lock_args.user_lock_hash {
            return Err(Error::InvalidOutputLockHash);
        }

        if req_cell.type_().is_none() {
            // Ckb -> SUDT
            let req_capcity = req_cell.capacity().unpack();
            let output_capcity = output_cell.capacity().unpack();
            let amount_in = req_capcity - SUDT_CAPACITY;

            if amount_in == 0 {
                return Err(Error::RequestCapcityEqSUDTCapcity);
            }

            if req_lock_args.sudt_type_hash != get_cell_type_hash!(index, Source::Output) {
                return Err(Error::InvalidOutputTypeHash);
            }

            if req_capcity <= output_capcity || req_capcity - output_capcity != amount_in {
                return Err(Error::InvalidCapacity);
            }

            if decode_u128(&load_cell_data(index, Source::Output)?)? < req_lock_args.min_amount_out
            {
                return Err(Error::SwapAmountLessThanMin);
            }
        } else {
            // SUDT -> Ckb
            let amount_in = decode_u128(&load_cell_data(index, Source::Input)?)?;

            if amount_in == 0 {
                return Err(Error::InputSUDTAmountEqZero);
            }

            if output_cell.type_().is_some() {
                return Err(Error::InvalidOutputTypeHash);
            }

            if BigUint::from(output_cell.capacity().unpack())
                < BigUint::from(req_cell.capacity().unpack()) + req_lock_args.min_amount_out
            {
                return Err(Error::InvalidCapacity);
            }

            if !load_cell_data(index, Source::Output)?.is_empty() {
                return Err(Error::InvalidOutputData);
            }
        }
    }

    Ok(())
}
