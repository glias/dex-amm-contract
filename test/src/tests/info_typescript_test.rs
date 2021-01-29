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

// #####################
// Initial Mint Tests
// #####################
test_contract!(
    initial_mint_success,
    {
        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let input_0 = Inputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .liquidity_sudt_type_hash(*LIQUIDITY_SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        let input_1 = Inputs::new_pool(SudtCell::new(POOL_BASE_CAPACITY, 0))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let liquidity_in_lock_args = LiquidityRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(3))
            .version(1)
            .sudt_min(0)
            .ckb_min(0)
            .info_type_hash(info_cell_type_hash(0))
            .tips(0)
            .tips_sudt(0)
            .build();

        println!("{:?}", hex::encode(liquidity_in_lock_args.as_bytes()));

        let input_3 = Inputs::new_liquidity(RequestCell::new(SUDT_CAPACITY + 50, 50))
            .custom_lock_args(liquidity_in_lock_args.as_bytes())
            .custom_type_args(liquidity_sudt_type_args());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(50)
                .sudt_reserve(50)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_BASE_CAPACITY + 50, 50))
            .custom_lock_args(Bytes::from(hash));
        let output_2 = Outputs::new_matcher(FreeCell::new(150));
        let output_3 =
            Outputs::new_sudt(SudtCell::new(100, 50)).custom_type_args(liquidity_sudt_type_args());

        let (mut context, tx) = build_test_context(vec![input_0, input_1, input_2, input_3], vec![
            output_0, output_1, output_2, output_3,
        ]);
        let tx = context.complete_tx(tx);

        context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

// #####################
// Mint Tests
// #####################
test_contract!(
    mint_liquidity_change_ckb_success,
    {
        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let input_0 = Inputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .total_liquidity(100)
                .sudt_reserve(50)
                .ckb_reserve(50)
                .liquidity_sudt_type_hash(*LIQUIDITY_SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        let input_1 = Inputs::new_pool(SudtCell::new(POOL_BASE_CAPACITY + 50, 50))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let liquidity_in_lock_args = LiquidityRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(9999))
            .version(1)
            .sudt_min(0)
            .ckb_min(30)
            .info_type_hash(info_cell_type_hash(0))
            .tips(0)
            .tips_sudt(0)
            .build();
        let input_3 = Inputs::new_liquidity(RequestCell::new(SUDT_CAPACITY + 100, 50))
            .custom_lock_args(liquidity_in_lock_args.as_bytes());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(101)
                .sudt_reserve(100)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_BASE_CAPACITY + 101, 100))
            .custom_lock_args(Bytes::from(hash));
        let output_2 = Outputs::new_matcher(FreeCell::new(150));
        let output_3 = Outputs::new_sudt(SudtCell::new(100, 101))
            .custom_type_args(liquidity_sudt_type_args())
            .custom_lock_args(Bytes::from(9999usize.to_le_bytes().to_vec()));
        let output_4 = Outputs::new_ckb(FreeCell::new(49))
            .custom_lock_args(Bytes::from(9999usize.to_le_bytes().to_vec()));

        let (mut context, tx) = build_test_context(vec![input_0, input_1, input_2, input_3], vec![
            output_0, output_1, output_2, output_3, output_4,
        ]);
        let tx = context.complete_tx(tx);

        context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

// Todo: not pass
test_contract!(
    mint_liquidity_change_sudt_success,
    {
        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(0).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let input_0 = Inputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .total_liquidity(100)
                .sudt_reserve(50)
                .ckb_reserve(50)
                .liquidity_sudt_type_hash(*LIQUIDITY_SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));

        // pool_in.capcity = POOL_BASE_CAPCITY + info_in.ckb_reserve
        // pool_in.amount = info_in.sudt_reserve
        let input_1 = Inputs::new_pool(SudtCell::new(POOL_BASE_CAPACITY + 50, 50))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let liquidity_in_lock_args = LiquidityRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(9999))
            .version(1)
            .sudt_min(80)
            .ckb_min(30)
            .info_type_hash(info_cell_type_hash(0))
            .tips(0)
            .tips_sudt(0)
            .build();
        let input_3 = Inputs::new_liquidity(RequestCell::new(SUDT_CAPACITY * 2 + 100, 302))
            .custom_lock_args(liquidity_in_lock_args.as_bytes());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(150)
                .sudt_reserve(151)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_BASE_CAPACITY + 101, 100))
            .custom_lock_args(Bytes::from(hash));
        let output_2 = Outputs::new_matcher(FreeCell::new(150));
        let output_3 = Outputs::new_sudt(SudtCell::new(100, 201))
            .custom_type_args(liquidity_sudt_type_args())
            .custom_lock_args(Bytes::from(9999usize.to_le_bytes().to_vec()));
        let output_4 = Outputs::new_sudt(SudtCell::new(50, 201))
            .custom_lock_args(Bytes::from(9999usize.to_le_bytes().to_vec()));

        let (mut context, tx) = build_test_context(vec![input_0, input_1, input_2, input_3], vec![
            output_0, output_1, output_2, output_3, output_4,
        ]);
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
