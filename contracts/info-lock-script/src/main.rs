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

use core::result::Result;

use share::ckb_std;
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    default_alloc,
    high_level::{load_cell, load_script, QueryIter},
};

use share::blake2b;
use share::get_cell_type_hash;

use error::Error;

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
    if QueryIter::new(load_cell, Source::GroupInput).count() != 2 {
        return Err(Error::InvalidInfoCellCount);
    }

    let pool_type_hash = get_cell_type_hash!(1, Source::Input);
    let self_args = load_script()?.args();
    let hash = blake2b!("ckb", pool_type_hash);

    if hash != self_args.as_slice()[0..32] {
        return Err(Error::InfoLockArgsFrontHalfMismatch);
    }

    if get_cell_type_hash!(0, Source::Input) != self_args.as_slice()[32..64] {
        return Err(Error::InfoLockArgsSecondHalfMismatch);
    }

    Ok(())
}
