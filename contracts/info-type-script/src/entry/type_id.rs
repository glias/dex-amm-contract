use alloc::vec::Vec;

use share::blake2b;
use share::ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    debug,
    high_level::{
        load_cell, load_cell_type_hash, load_script, load_script_hash, load_transaction, QueryIter,
    },
};

use crate::error::Error;

pub fn verify_type_id() -> Result<(), Error> {
    // TYPE_ID script should only accept one argument,
    // which is the hash of all inputs when creating
    // the cell.
    let script_args: Vec<u8> = load_script()?.args().unpack();
    if script_args.len() != 32 {
        return Err(Error::InvalidInfoTypeArgsLen);
    }

    // There could be at most one input cell and one
    // output cell with current TYPE_ID script.
    if QueryIter::new(load_cell, Source::GroupInput).count() > 1
        || QueryIter::new(load_cell, Source::GroupOutput).count() > 1
    {
        return Err(Error::InputCellMoreThanOne);
    }

    // If there's only one output cell with current
    // TYPE_ID script, we are creating such a cell,
    // we also need to validate that the first argument matches
    // the hash of following items concatenated:
    // 1. Transaction hash of the first CellInput's OutPoint
    // 2. Index of the first output cell in current script group.
    let self_hash = load_script_hash()?;
    if QueryIter::new(load_cell, Source::GroupOutput).count() == 1 {
        let first_cell_input_outpoint = load_transaction()?
            .raw()
            .inputs()
            .get(0)
            .unwrap()
            .previous_output();

        let first_output_index = QueryIter::new(load_cell_type_hash, Source::Output)
            .enumerate()
            .find_map(|(idx, hash)| {
                if hash == Some(self_hash) {
                    Some(idx)
                } else {
                    None
                }
            })
            .unwrap() as u64;

        let hash = blake2b!(
            first_cell_input_outpoint.tx_hash().unpack(),
            first_output_index.to_le_bytes()
        );

        debug!("{:?}", script_args);
        if hash[..] != script_args[..] {
            return Err(Error::InvalidTypeID);
        }
    }

    Ok(())
}
