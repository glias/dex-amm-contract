mod info_lockscript_test;
mod info_typescript_test;
mod liquidity_lockscript_test;
mod swap_lockscript_test;

use std::collections::HashMap;

use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::context::Context;
use ckb_tool::ckb_error::assert_error_eq;
use ckb_tool::ckb_types::bytes::Bytes;
use ckb_tool::ckb_types::packed::*;
use ckb_tool::ckb_types::prelude::*;
use ckb_x64_simulator::RunningSetup;
use molecule::prelude::*;
use share::blake2b;

use crate::{cell_builder::*, tx_builder::*};
use crate::{test_contract, Loader};

const MAX_CYCLES: u64 = 10000_0000;
const POOL_CAPACITY: u64 = 18_600_000_000;
const SUDT_CAPACITY: u64 = 14_200_000_000;
const INFO_CAPACITY: u64 = 25_000_000_000;

lazy_static::lazy_static! {
    static ref SUDT_TYPE_HASH: [u8; 32] = {
        let mut ctx = Context::default();
        let always_success_out_point = ctx.deploy_cell(ALWAYS_SUCCESS.clone());
        ctx.build_script(&always_success_out_point, Default::default())
            .unwrap()
            .calc_script_hash()
            .unpack()
    };
    static ref LIQUIDITY_SUDT_TYPE_HASH: [u8; 32] = {
        let mut ctx = Context::default();
        let args = Bytes::from(9999u64.to_le_bytes().to_vec());
        let always_success_out_point = ctx.deploy_cell(ALWAYS_SUCCESS.clone());
        ctx.build_script(&always_success_out_point, args)
            .unwrap()
            .calc_script_hash()
            .unpack()
    };
    static ref INFO_TYPE_SCRIPT: Bytes = Loader::default().load_binary("info-type-script");
    static ref INFO_LOCK_SCRIPT: Bytes = Loader::default().load_binary("info-lock-script");
}

#[macro_export]
macro_rules! test_contract {
    ($case_name:ident, $body:expr, $is_lockscript: expr, $sim_name: expr) => {
        #[test]
        fn $case_name() {
            let (context, tx) = $body;

            let setup = RunningSetup {
                is_lock_script:  $is_lockscript,
                is_output:       false,
                script_index:    0,
                native_binaries: HashMap::default(),
            };

            write_native_setup(stringify!($case_name), $sim_name, &tx, &context, &setup);
        }
    };
}

fn info_cell_type_hash(idx: usize) -> [u8; 32] {
    let args = info_type_args(idx);
    Script::new_builder()
        .code_hash(CellOutput::calc_data_hash(&INFO_TYPE_SCRIPT))
        .hash_type(Byte::new(0))
        .args(args.pack())
        .build()
        .calc_script_hash()
        .unpack()
}

fn user_lock_hash(idx: usize) -> [u8; 32] {
    let mut ctx = Context::default();
    let always_success_out_point = ctx.deploy_cell(ALWAYS_SUCCESS.clone());
    let args = Bytes::from(idx.to_le_bytes().to_vec());
    ctx.build_script(&always_success_out_point, args)
        .unwrap()
        .calc_script_hash()
        .unpack()
}

fn info_type_args(idx: usize) -> Bytes {
    let mut ctx = Context::default();
    let always_success_out_point = ctx.deploy_cell(ALWAYS_SUCCESS.clone());
    let args = Bytes::from(idx.to_le_bytes().to_vec());
    ctx.build_script(&always_success_out_point, args)
        .unwrap()
        .calc_script_hash()
        .as_bytes()
}

fn sudt_input_out_point(
    capacity: u64,
    lock_args: Bytes,
    type_args: Option<Bytes>,
    data: Bytes,
) -> OutPoint {
    let mut ctx = Context::default();
    let always_success_out_point = ctx.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = ctx
        .build_script(&always_success_out_point, lock_args)
        .unwrap();

    let type_script = ctx
        .build_script(&always_success_out_point, type_args.unwrap_or_default())
        .unwrap();

    ctx.create_cell(
        CellOutput::new_builder()
            .capacity(capacity.pack())
            .type_(Some(type_script).pack())
            .lock(lock_script)
            .build(),
        data,
    )
}

fn liquidity_sudt_type_args() -> Bytes {
    Bytes::from(9999usize.to_le_bytes().to_vec())
}

fn user_lock_args(idx: usize) -> Bytes {
    Bytes::from(idx.to_le_bytes().to_vec())
}

fn witness_args_input_type(num: u64) -> Bytes {
    let byte_opt = Some(Bytes::from(num.to_le_bytes().to_vec())).pack();
    WitnessArgsBuilder::default()
        .input_type(byte_opt)
        .build()
        .as_bytes()
}
