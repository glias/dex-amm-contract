use super::*;

const ERR_OUTPUT_INFO_LOCK_ARGS_FIRST_HALF_DIFF: i8 = 30;
const ERR_OUTPUT_INFO_LOCK_ARGS_SECOND_HALF_DIFF: i8 = 31;
const ERR_OUTPUT_CELL_WITH_INFO_LOCK_HASH_MORE_THAN_TWO: i8 = 32;
const ERR_OUTPUT_POOL_CELL_DATA_LEN_TOO_SHORT: i8 = 34;
const ERR_OUTPUT_CELLS_LOCK_HASH_DIFF: i8 = 35;

// #####################
// Pool Creation Tests
// #####################
test_contract!(
    info_creation_success,
    {
        let input = Inputs::new_sudt(SudtCell::new(21000, 1500));

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .ckb_reserve(500)
                .sudt_reserve(500)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        let output_1 =
            Outputs::new_pool(SudtCell::new(21000, 1500)).custom_lock_args(Bytes::from(hash));

        let (mut context, tx) = build_test_context(vec![input], vec![output_0, output_1]);
        let tx = context.complete_tx(tx);

        // let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        // assert_error_eq!(err, tx_error(, 0));

        context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    info_creation_output_three_cell_with_info_lock_hash,
    {
        let input = Inputs::new_sudt(SudtCell::new(21000, 1500));

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .ckb_reserve(500)
                .sudt_reserve(500)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        let output_1 = Outputs::new_pool(SudtCell::new(21000, 1500))
            .custom_lock_args(Bytes::from(hash.clone()));

        let output_2 =
            Outputs::new_pool(SudtCell::new(31000, 2500)).custom_lock_args(Bytes::from(hash));

        let (mut context, tx) = build_test_context(vec![input], vec![output_0, output_1, output_2]);
        let tx = context.complete_tx(tx);

        let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        assert_error_eq!(
            err,
            tx_error(
                ERR_OUTPUT_CELL_WITH_INFO_LOCK_HASH_MORE_THAN_TWO,
                0,
                false,
                false
            )
        );

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    info_creation_info_out_lock_args_first_half_diff,
    {
        let input = Inputs::new_sudt(SudtCell::new(21000, 1500));

        let mut hash = blake2b!("sckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .ckb_reserve(500)
                .sudt_reserve(500)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        let output_1 = Outputs::new_pool(SudtCell::new(21000, 1500))
            .custom_lock_args(Bytes::from(hash.clone()));

        let (mut context, tx) = build_test_context(vec![input], vec![output_0, output_1]);
        let tx = context.complete_tx(tx);

        let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        assert_error_eq!(
            err,
            tx_error(ERR_OUTPUT_INFO_LOCK_ARGS_FIRST_HALF_DIFF, 0, false, false)
        );

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    info_creation_info_out_lock_args_second_half_diff,
    {
        let input = Inputs::new_sudt(SudtCell::new(21000, 1500));

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash_1.reverse();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .ckb_reserve(500)
                .sudt_reserve(500)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        let output_1 = Outputs::new_pool(SudtCell::new(21000, 1500))
            .custom_lock_args(Bytes::from(hash.clone()));

        let (mut context, tx) = build_test_context(vec![input], vec![output_0, output_1]);
        let tx = context.complete_tx(tx);

        let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        assert_error_eq!(
            err,
            tx_error(ERR_OUTPUT_INFO_LOCK_ARGS_SECOND_HALF_DIFF, 0, false, false)
        );

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    info_creation_output_cells_lock_hash_diff,
    {
        let input = Inputs::new_sudt(SudtCell::new(21000, 1500));

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .ckb_reserve(500)
                .sudt_reserve(500)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        let output_1 = Outputs::new_pool(SudtCell::new(21000, 1500))
            .custom_lock_args(Bytes::from(&b"changed_lock_hash"[..]));

        let (mut context, tx) = build_test_context(vec![input], vec![output_0, output_1]);
        let tx = context.complete_tx(tx);

        let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        assert_error_eq!(
            err,
            tx_error(ERR_OUTPUT_CELLS_LOCK_HASH_DIFF, 0, false, false)
        );

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    info_creation_pool_cell_data_too_short,
    {
        let input = Inputs::new_sudt(SudtCell::new(21000, 1500));

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .ckb_reserve(500)
                .sudt_reserve(500)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        let output_1 =
            Outputs::new_pool(SudtCell::new_unchecked(21000, Bytes::from(vec![0, 1, 2])))
                .custom_lock_args(Bytes::from(hash.clone()));

        let (mut context, tx) = build_test_context(vec![input], vec![output_0, output_1]);
        let tx = context.complete_tx(tx);

        let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        assert_error_eq!(
            err,
            tx_error(ERR_OUTPUT_POOL_CELL_DATA_LEN_TOO_SHORT, 0, false, false)
        );

        (context, tx)
    },
    false,
    "info-typescript-sim"
);
