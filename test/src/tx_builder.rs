use std::fs;
use std::path::PathBuf;

use ckb_dyn_lock::locks::binary::{self, Binary};
use ckb_standalone_debugger::transaction::{
    MockCellDep, MockInfo, MockInput, MockTransaction, ReprMockTransaction,
};
// use ckb_system_scripts::BUNDLED_CELL;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_crypto::secp::Pubkey;
// use ckb_tool::ckb_hash::{blake2b_256, new_blake2b};
use ckb_tool::ckb_script::{ScriptError, TransactionScriptError};
use ckb_tool::ckb_types::core::{DepType, TransactionBuilder, TransactionView};
use ckb_tool::ckb_types::packed::*;
use ckb_tool::ckb_types::{bytes::Bytes, prelude::*};
use ckb_x64_simulator::RunningSetup;
use molecule::prelude::*;
use serde_json::to_string_pretty;

use crate::cell_builder::{FreeCell, InfoCell, LiquidityRequestCell, SudtCell, SwapRequestCell};
use crate::{Loader, TX_FOLDER};

pub enum InputCell {
    Sudt(SudtCell),
    Info(InfoCell),
    Matcher(FreeCell),
    Liquidity(LiquidityRequestCell),
    Swap(SwapRequestCell),
    Pool(SudtCell),
}

pub enum OutputCell {
    Info(InfoCell),
    Sudt(SudtCell),
    Ckb(FreeCell),
    Matcher(FreeCell),
    Pool(SudtCell),
}

pub struct Inputs {
    cell:             InputCell,
    cell_deps:        Option<Vec<CellDep>>,
    custom_type_args: Option<Bytes>,
    custom_lock_args: Option<Bytes>,
    witness:          Option<Bytes>,
}

impl Inputs {
    pub fn new_info(cell: InfoCell) -> Self {
        Self::inner_new(InputCell::Info(cell))
    }

    pub fn new_sudt(cell: SudtCell) -> Self {
        Self::inner_new(InputCell::Sudt(cell))
    }

    pub fn new_matcher(cell: FreeCell) -> Self {
        Self::inner_new(InputCell::Matcher(cell))
    }

    pub fn new_pool(cell: SudtCell) -> Self {
        Self::inner_new(InputCell::Pool(cell))
    }

    pub fn new_liquidity(cell: LiquidityRequestCell) -> Self {
        Self::inner_new(InputCell::Liquidity(cell))
    }

    pub fn new_swap(cell: SwapRequestCell) -> Self {
        Self::inner_new(InputCell::Swap(cell))
    }

    fn inner_new(cell: InputCell) -> Self {
        Inputs {
            cell,
            cell_deps: None,
            custom_type_args: None,
            custom_lock_args: None,
            witness: None,
        }
    }

    pub fn custom_type_args(mut self, args: Bytes) -> Self {
        self.custom_type_args = Some(args);
        self
    }

    pub fn custom_lock_args(mut self, args: Bytes) -> Self {
        self.custom_lock_args = Some(args);
        self
    }

    pub fn custom_witness(mut self, witness: Bytes) -> Self {
        self.witness = Some(witness);
        self
    }

    pub fn custom_cell_deps(mut self, cell_deps: Vec<CellDep>) -> Self {
        self.cell_deps = Some(cell_deps);
        self
    }
}

pub struct Outputs {
    cell:             OutputCell,
    custom_type_args: Option<Bytes>,
    custom_lock_args: Option<Bytes>,
}

impl Outputs {
    pub fn new_info(cell: InfoCell) -> Self {
        Self::inner_new(OutputCell::Info(cell))
    }

    pub fn new_sudt(cell: SudtCell) -> Self {
        Self::inner_new(OutputCell::Sudt(cell))
    }

    pub fn new_ckb(cell: FreeCell) -> Self {
        Self::inner_new(OutputCell::Ckb(cell))
    }

    pub fn new_matcher(cell: FreeCell) -> Self {
        Self::inner_new(OutputCell::Matcher(cell))
    }

    pub fn new_pool(cell: SudtCell) -> Self {
        Self::inner_new(OutputCell::Pool(cell))
    }

    fn inner_new(cell: OutputCell) -> Self {
        Outputs {
            cell,
            custom_type_args: None,
            custom_lock_args: None,
        }
    }

    pub fn custom_type_args(mut self, args: Bytes) -> Self {
        self.custom_type_args = Some(args);
        self
    }

    pub fn custom_lock_args(mut self, args: Bytes) -> Self {
        self.custom_lock_args = Some(args);
        self
    }
}

fn build_tx(
    context: &mut Context,
    input_orders: Vec<Inputs>,
    output_results: Vec<Outputs>,
) -> TransactionView {
    let info_lock_bin: Bytes = Loader::default().load_binary("info-lock-script");
    let info_lock_out_point = context.deploy_cell(info_lock_bin);
    let info_lock_dep = CellDep::new_builder()
        .out_point(info_lock_out_point.clone())
        .build();

    let info_type_bin: Bytes = Loader::default().load_binary("info-type-script");
    let info_type_out_point = context.deploy_cell(info_type_bin);
    let info_type_dep = CellDep::new_builder()
        .out_point(info_type_out_point.clone())
        .build();

    let liquidity_lock_bin: Bytes = Loader::default().load_binary("liquidity-lock-script");
    let liquidity_lock_out_point = context.deploy_cell(liquidity_lock_bin);
    let liquidity_lock_dep = CellDep::new_builder()
        .out_point(liquidity_lock_out_point.clone())
        .build();

    let swap_lock_bin: Bytes = Loader::default().load_binary("swap-lock-script");
    let swap_lock_out_point = context.deploy_cell(swap_lock_bin);
    let swap_lock_dep = CellDep::new_builder()
        .out_point(swap_lock_out_point.clone())
        .build();

    // Deploy always sucess script
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let always_success_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();

    // Always success lock script
    let always_success_lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("always success lock script");

    // Use always success as test sudt type contract
    let sudt_type_script = always_success_lock_script;

    // Pass idx as args to always success lock script to mock different user lock script
    let create_user_lock_script = |context: &mut Context, idx: usize| -> (Script, Bytes) {
        let user_lock_script = {
            let args = Bytes::from(idx.to_le_bytes().to_vec());
            context
                .build_script(&always_success_out_point, args)
                .expect("user lock script")
        };
        let hash = user_lock_script.calc_script_hash().as_bytes();
        (user_lock_script, hash)
    };

    // Prepare inputs
    let mut inputs = vec![];
    let mut witnesses = vec![];
    let mut cell_deps: Vec<CellDep> = vec![];
    for (idx, input) in input_orders.into_iter().enumerate() {
        let (user_lock_script, hash) = create_user_lock_script(context, idx);

        let user_lock_script = match input.custom_lock_args.clone() {
            Some(lock_args) => user_lock_script.as_builder().args(lock_args.pack()).build(),
            None => user_lock_script,
        };

        let sudt_type_script = match input.custom_type_args.clone() {
            Some(type_args) => {
                let type_script = sudt_type_script.clone();
                type_script.as_builder().args(type_args.pack()).build()
            }
            None => sudt_type_script.clone(),
        };

        match input.cell {
            InputCell::Info(cell) => {
                let lock_args = input.custom_lock_args.expect("info input lock args");

                let info_lock_script = context
                    .build_script(&info_lock_out_point, lock_args)
                    .expect("info lock script");

                let info_type_script = context
                    .build_script(&info_type_out_point, hash.clone())
                    .expect("info type script");

                let input_out_point = context.create_cell(
                    CellOutput::new_builder()
                        .capacity(cell.capacity.pack())
                        .lock(info_lock_script.clone())
                        .type_(Some(info_type_script).pack())
                        .build(),
                    cell.data,
                );

                let input_cell = CellInput::new_builder()
                    .previous_output(input_out_point)
                    .build();

                cell_deps.extend(input.cell_deps.unwrap_or_default());
                inputs.push(input_cell);
                witnesses.push(input.witness.unwrap_or_default());
            }
            InputCell::Sudt(cell) => {
                let input_out_point = if let Some(out_point) = cell.out_point {
                    context.create_cell_with_out_point(
                        out_point.clone(),
                        CellOutput::new_builder()
                            .capacity(cell.capacity.pack())
                            .type_(Some(sudt_type_script.clone()).pack())
                            .lock(user_lock_script)
                            .build(),
                        cell.data,
                    );
                    out_point
                } else {
                    context.create_cell(
                        CellOutput::new_builder()
                            .capacity(cell.capacity.pack())
                            .type_(Some(sudt_type_script.clone()).pack())
                            .lock(user_lock_script)
                            .build(),
                        cell.data,
                    )
                };

                let input_cell = CellInput::new_builder()
                    .previous_output(input_out_point)
                    .build();

                cell_deps.extend(input.cell_deps.unwrap_or_default());
                inputs.push(input_cell);
                witnesses.push(input.witness.unwrap_or_default());
            }
            InputCell::Pool(cell) => {
                let lock_args = input.custom_lock_args.expect("pool input lock args");
                let info_lock_script = context
                    .build_script(&info_lock_out_point, lock_args)
                    .expect("info lock script");
                let input_out_point = context.create_cell(
                    CellOutput::new_builder()
                        .capacity(cell.capacity.pack())
                        .type_(Some(sudt_type_script.clone()).pack())
                        .lock(info_lock_script)
                        .build(),
                    cell.data,
                );

                let input_cell = CellInput::new_builder()
                    .previous_output(input_out_point)
                    .build();

                cell_deps.extend(input.cell_deps.unwrap_or_default());
                inputs.push(input_cell);
                witnesses.push(input.witness.unwrap_or_default());
            }
            InputCell::Matcher(cell) => {
                let input_out_point = context.create_cell(
                    CellOutput::new_builder()
                        .capacity(cell.capacity.pack())
                        .lock(user_lock_script)
                        .build(),
                    Bytes::new(),
                );

                let input_cell = CellInput::new_builder()
                    .previous_output(input_out_point)
                    .build();

                cell_deps.extend(input.cell_deps.unwrap_or_default());
                inputs.push(input_cell);
                witnesses.push(input.witness.unwrap_or_default());
            }
            InputCell::Liquidity(cell) => {
                let lock_args = input.custom_lock_args.expect("liquidity input lock args");
                let liquidity_lock = context
                    .build_script(&liquidity_lock_out_point, lock_args)
                    .expect("liquidity lock script");

                let input_out_point = context.create_cell(
                    CellOutput::new_builder()
                        .capacity(cell.capacity.pack())
                        .lock(liquidity_lock)
                        .type_(Some(sudt_type_script).pack())
                        .build(),
                    cell.data,
                );

                let input_cell = CellInput::new_builder()
                    .previous_output(input_out_point)
                    .build();

                cell_deps.extend(input.cell_deps.unwrap_or_default());
                inputs.push(input_cell);
                witnesses.push(input.witness.unwrap_or_default());
            }
            InputCell::Swap(cell) => {
                let lock_args = input.custom_lock_args.expect("swap input lock args");
                let swap_lock = context
                    .build_script(&swap_lock_out_point, lock_args)
                    .expect("swap lock script");

                let type_script = if cell.is_sudt {
                    Some(sudt_type_script)
                } else {
                    None
                };

                let input_out_point = context.create_cell(
                    CellOutput::new_builder()
                        .capacity(cell.capacity.pack())
                        .lock(swap_lock)
                        .type_(type_script.pack())
                        .build(),
                    cell.data,
                );

                let input_cell = CellInput::new_builder()
                    .previous_output(input_out_point)
                    .build();

                cell_deps.extend(input.cell_deps.unwrap_or_default());
                inputs.push(input_cell);
                witnesses.push(input.witness.unwrap_or_default());
            }
        }
    }

    let mut outputs = vec![];
    let mut outputs_data = vec![];
    for (idx, output) in output_results.into_iter().enumerate() {
        let (user_lock_script, hash) = create_user_lock_script(context, idx);

        let user_lock_script = match output.custom_lock_args.clone() {
            Some(lock_args) => user_lock_script.as_builder().args(lock_args.pack()).build(),
            None => user_lock_script,
        };

        let sudt_type_script = match output.custom_type_args.clone() {
            Some(type_args) => {
                let type_script = sudt_type_script.clone();
                type_script.as_builder().args(type_args.pack()).build()
            }
            None => sudt_type_script.clone(),
        };

        let (output, data) = match output.cell {
            OutputCell::Info(cell) => {
                let args = output.custom_lock_args.expect("info out lock args");
                let info_lock_script = context
                    .build_script(&info_lock_out_point, args)
                    .expect("info lock script");
                let info_type_script = context
                    .build_script(&info_type_out_point, hash)
                    .expect("info type script");

                let output = CellOutput::new_builder()
                    .capacity(cell.capacity.pack())
                    .type_(Some(info_type_script).pack())
                    .lock(info_lock_script)
                    .build();

                (output, cell.data)
            }
            OutputCell::Sudt(cell) => {
                let output = CellOutput::new_builder()
                    .capacity(cell.capacity.pack())
                    .type_(Some(sudt_type_script).pack())
                    .lock(user_lock_script)
                    .build();

                (output, cell.data)
            }
            OutputCell::Ckb(cell) => {
                let output = CellOutput::new_builder()
                    .capacity(cell.capacity.pack())
                    .lock(user_lock_script)
                    .build();

                (output, cell.data)
            }
            OutputCell::Matcher(cell) => {
                let output = CellOutput::new_builder()
                    .capacity(cell.capacity.pack())
                    .lock(user_lock_script)
                    .build();

                (output, cell.data)
            }
            OutputCell::Pool(cell) => {
                let lock_args = output.custom_lock_args.expect("pool output lock args");
                let info_lock_script = context
                    .build_script(&info_lock_out_point, lock_args)
                    .expect("info lock script");
                let output = CellOutput::new_builder()
                    .capacity(cell.capacity.pack())
                    .type_(Some(sudt_type_script).pack())
                    .lock(info_lock_script)
                    .build();

                (output, cell.data)
            }
        };

        outputs.push(output);
        outputs_data.push(data);
    }

    TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(info_lock_dep)
        .cell_dep(info_type_dep)
        .cell_dep(liquidity_lock_dep)
        .cell_dep(swap_lock_dep)
        .cell_dep(always_success_dep)
        .cell_deps(cell_deps)
        .witnesses(witnesses.pack())
        .build()
}

pub fn build_test_context(
    input_orders: Vec<Inputs>,
    output_results: Vec<Outputs>,
) -> (Context, TransactionView) {
    let mut context = Context::default();
    let tx = build_tx(&mut context, input_orders, output_results);
    (context, tx)
}

pub fn tx_error(
    error_code: i8,
    index: usize,
    is_input: bool,
    is_lockscript: bool,
) -> TransactionScriptError {
    if is_input && is_lockscript {
        ScriptError::ValidationFailure(error_code).input_lock_script(index)
    } else if is_input && !is_lockscript {
        ScriptError::ValidationFailure(error_code).input_type_script(index)
    } else if !is_input && !is_lockscript {
        ScriptError::ValidationFailure(error_code).output_type_script(index)
    } else {
        unreachable!()
    }
}

struct DynLock;

impl DynLock {
    fn deploy(context: &mut Context) -> (OutPoint, Vec<CellDep>) {
        let secp256k1_data_bin = binary::get(Binary::Secp256k1Data);
        let secp256k1_data_out_point = context.deploy_cell(secp256k1_data_bin.to_vec().into());
        let secp256k1_data_dep = CellDep::new_builder()
            .out_point(secp256k1_data_out_point)
            .build();

        let secp256k1_keccak256_bin = binary::get(Binary::Secp256k1Keccak256SighashAllDual);
        let secp256k1_keccak256_out_point =
            context.deploy_cell(secp256k1_keccak256_bin.to_vec().into());
        let secp256k1_keccak256_dep = CellDep::new_builder()
            .out_point(secp256k1_keccak256_out_point.clone())
            .build();

        (secp256k1_keccak256_out_point, vec![
            secp256k1_data_dep,
            secp256k1_keccak256_dep,
        ])
    }

    fn eth_pubkey(pubkey: Pubkey) -> Bytes {
        use sha3::{Digest, Keccak256};

        let prefix_key: [u8; 65] = {
            let mut temp = [4u8; 65];
            temp[1..65].copy_from_slice(pubkey.as_bytes());
            temp
        };
        let pubkey = secp256k1::key::PublicKey::from_slice(&prefix_key).unwrap();
        let message = Vec::from(&pubkey.serialize_uncompressed()[1..]);

        let mut hasher = Keccak256::default();
        hasher.input(&message);
        Bytes::copy_from_slice(&hasher.result()[12..32])
    }
}

fn create_test_folder(name: &str) -> PathBuf {
    let mut path = TX_FOLDER.clone();
    path.push(&name);
    fs::create_dir_all(&path).expect("create folder");
    path
}

fn build_mock_transaction(tx: &TransactionView, context: &Context) -> MockTransaction {
    let mock_inputs = tx
        .inputs()
        .into_iter()
        .map(|input| {
            let (output, data) = context
                .get_cell(&input.previous_output())
                .expect("get cell");
            MockInput {
                input,
                output,
                data,
                header: None,
            }
        })
        .collect();
    let mock_cell_deps = tx
        .cell_deps()
        .into_iter()
        .map(|cell_dep| {
            if cell_dep.dep_type() == DepType::DepGroup.into() {
                panic!("Implement dep group support later!");
            }
            let (output, data) = context.get_cell(&cell_dep.out_point()).expect("get cell");
            MockCellDep {
                cell_dep,
                output,
                data,
                header: None,
            }
        })
        .collect();
    let mock_info = MockInfo {
        inputs:      mock_inputs,
        cell_deps:   mock_cell_deps,
        header_deps: vec![],
    };
    MockTransaction {
        mock_info,
        tx: tx.data(),
    }
}

pub fn write_native_setup(
    test_name: &str,
    binary_name: &str,
    tx: &TransactionView,
    context: &Context,
    setup: &RunningSetup,
) {
    let folder = create_test_folder(test_name);
    let mock_tx = build_mock_transaction(&tx, &context);
    let repr_tx: ReprMockTransaction = mock_tx.into();
    let tx_json = to_string_pretty(&repr_tx).expect("serialize to json");
    fs::write(folder.join("tx.json"), tx_json).expect("write tx to local file");
    let setup_json = to_string_pretty(setup).expect("serialize to json");
    fs::write(folder.join("setup.json"), setup_json).expect("write setup to local file");
    fs::write(
        folder.join("cmd"),
        format!(
            "CKB_TX_FILE=\"{}\" CKB_RUNNING_SETUP=\"{}\" \"{}\"",
            folder.join("tx.json").to_str().expect("utf8"),
            folder.join("setup.json").to_str().expect("utf8"),
            Loader::default().path(binary_name).to_str().expect("utf8")
        ),
    )
    .expect("write cmd to local file");
}

// struct Secp256k1Lock;

// impl Secp256k1Lock {
//     fn deploy(context: &mut Context) -> (OutPoint, Vec<CellDep>) {
//         let secp256k1_lock_bin = BUNDLED_CELL
//             .get("specs/cells/secp256k1_blake160_sighash_all")
//             .unwrap();
//         let secp256k1_lock_out_point = context.deploy_cell(secp256k1_lock_bin.to_vec().into());
//         let secp256k1_lock_dep = CellDep::new_builder()
//             .out_point(secp256k1_lock_out_point.clone())
//             .build();

//         let secp256k1_data_bin = BUNDLED_CELL.get("specs/cells/secp256k1_data").unwrap();
//         let secp256k1_data_out_point = context.deploy_cell(secp256k1_data_bin.to_vec().into());
//         let secp256k1_data_dep = CellDep::new_builder()
//             .out_point(secp256k1_data_out_point)
//             .build();

//         (secp256k1_lock_out_point, vec![
//             secp256k1_lock_dep,
//             secp256k1_data_dep,
//         ])
//     }

//     fn sign_tx(tx: TransactionView, key: &Privkey) -> TransactionView {
//         const SIGNATURE_SIZE: usize = 65;

//         let witnesses_len = tx.inputs().len();
//         let tx_hash = tx.hash();
//         let mut signed_witnesses: Vec<packed::Bytes> = Vec::new();
//         let mut blake2b = new_blake2b();
//         let mut message = [0u8; 32];
//         blake2b.update(&tx_hash.raw_data());

//         // digest the first witness
//         let witness = WitnessArgs::default();
//         let zero_lock: Bytes = {
//             let mut buf = Vec::new();
//             buf.resize(SIGNATURE_SIZE, 0);
//             buf.into()
//         };
//         let witness_for_digest = witness
//             .clone()
//             .as_builder()
//             .lock(Some(zero_lock).pack())
//             .build();

//         let witness_len = witness_for_digest.as_bytes().len() as u64;
//         blake2b.update(&witness_len.to_le_bytes());
//         blake2b.update(&witness_for_digest.as_bytes());
//         blake2b.finalize(&mut message);
//         let message = H256::from(message);
//         let sig = key.sign_recoverable(&message).expect("sign");
//         signed_witnesses.push(
//             witness
//                 .clone()
//                 .as_builder()
//                 .lock(Some(Bytes::from(sig.serialize())).pack())
//                 .build()
//                 .as_bytes()
//                 .pack(),
//         );
//         for i in 1..witnesses_len {
//             signed_witnesses.push(tx.witnesses().get(i).unwrap());
//         }
//         tx.as_advanced_builder()
//             .set_witnesses(signed_witnesses)
//             .build()
//     }

//     fn blake160(data: &[u8]) -> [u8; 20] {
//         let mut buf = [0u8; 20];
//         let hash = blake2b_256(data);
//         buf.clone_from_slice(&hash[..20]);
//         buf
//     }
// }
