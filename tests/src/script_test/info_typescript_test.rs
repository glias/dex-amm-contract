use super::*;

const ERR_OUTPUT_INFO_LOCK_ARGS_FIRST_HALF_DIFF: i8 = 30;
const ERR_OUTPUT_INFO_LOCK_ARGS_SECOND_HALF_DIFF: i8 = 31;
const ERR_INVALID_INFO_LOCK_COUNT_IN_OUTPUT: i8 = 33;
const ERR_OUTPUT_POOL_CELL_DATA_LEN_TOO_SHORT: i8 = 34;
const ERR_OUTPUT_CELLS_LOCK_HASH_DIFF: i8 = 35;

// #####################
// Pool Creation Tests
// #####################
test_contract!(
    info_creation_success,
    {
        let sudt_data: Uint128 = 1500u128.pack();
        let input_out_point =
            sudt_input_out_point(21000, user_lock_args(0), None, sudt_data.as_bytes());
        let input_out_point_tx_hash: [u8; 32] = input_out_point.tx_hash().unpack();

        let input = Inputs::new_sudt(SudtCell::new_with_out_point(21000, 1500, input_out_point));

        let hash = blake2b!(input_out_point_tx_hash, 0u64.to_le_bytes());
        let type_id = Bytes::from(hash.to_vec());

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(type_id.clone()).to_vec();
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
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_type_args(type_id);

        let output_1 =
            Outputs::new_pool(SudtCell::new(21000, 1500)).custom_lock_args(Bytes::from(hash));

        let (mut context, tx) = build_test_context(vec![input], vec![output_0, output_1]);
        let tx = context.complete_tx(tx);

        // let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        // assert_error_eq!(err, tx_error(, 0));

        let cycle = context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        println!("cycle used {:?}", cycle);

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    info_creation_data_deploy_output_three_cell_with_info_lock_hash,
    {
        let sudt_data: Uint128 = 1500u128.pack();
        let input_out_point =
            sudt_input_out_point(21000, user_lock_args(0), None, sudt_data.as_bytes());
        let input_out_point_tx_hash: [u8; 32] = input_out_point.tx_hash().unpack();

        let input = Inputs::new_sudt(SudtCell::new_with_out_point(21000, 1500, input_out_point));

        let hash = blake2b!(input_out_point_tx_hash, 0u64.to_le_bytes());
        let type_id = Bytes::from(hash.to_vec());

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(type_id.clone()).to_vec();
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
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_type_args(type_id);

        let output_1 = Outputs::new_pool(SudtCell::new(21000, 1500))
            .custom_lock_args(Bytes::from(hash.clone()));

        let output_2 =
            Outputs::new_pool(SudtCell::new(31000, 2500)).custom_lock_args(Bytes::from(hash));

        let (mut context, tx) = build_test_context(vec![input], vec![output_0, output_1, output_2]);
        let tx = context.complete_tx(tx);

        let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        assert_error_eq!(
            err,
            tx_error(ERR_INVALID_INFO_LOCK_COUNT_IN_OUTPUT, 0, false, false)
        );

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    info_creation_info_out_lock_args_first_half_diff,
    {
        let sudt_data: Uint128 = 1500u128.pack();
        let input_out_point =
            sudt_input_out_point(21000, user_lock_args(0), None, sudt_data.as_bytes());
        let input_out_point_tx_hash: [u8; 32] = input_out_point.tx_hash().unpack();

        let input = Inputs::new_sudt(SudtCell::new_with_out_point(21000, 1500, input_out_point));

        let hash = blake2b!(input_out_point_tx_hash, 0u64.to_le_bytes());
        let type_id = Bytes::from(hash.to_vec());

        let mut hash = blake2b!("sckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(type_id.clone()).to_vec();
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
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_type_args(type_id);

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
        let sudt_data: Uint128 = 1500u128.pack();
        let input_out_point =
            sudt_input_out_point(21000, user_lock_args(0), None, sudt_data.as_bytes());
        let input_out_point_tx_hash: [u8; 32] = input_out_point.tx_hash().unpack();

        let input = Inputs::new_sudt(SudtCell::new_with_out_point(21000, 1500, input_out_point));

        let hash = blake2b!(input_out_point_tx_hash, 0u64.to_le_bytes());
        let type_id = Bytes::from(hash.to_vec());

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(type_id.clone()).to_vec();
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
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_type_args(type_id);

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
        let sudt_data: Uint128 = 1500u128.pack();
        let input_out_point =
            sudt_input_out_point(21000, user_lock_args(0), None, sudt_data.as_bytes());
        let input_out_point_tx_hash: [u8; 32] = input_out_point.tx_hash().unpack();

        let input = Inputs::new_sudt(SudtCell::new_with_out_point(21000, 1500, input_out_point));

        let hash = blake2b!(input_out_point_tx_hash, 0u64.to_le_bytes());
        let type_id = Bytes::from(hash.to_vec());

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(type_id.clone()).to_vec();
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
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_type_args(type_id);

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
        let sudt_data: Uint128 = 1500u128.pack();
        let input_out_point =
            sudt_input_out_point(21000, user_lock_args(0), None, sudt_data.as_bytes());
        let input_out_point_tx_hash: [u8; 32] = input_out_point.tx_hash().unpack();

        let input = Inputs::new_sudt(SudtCell::new_with_out_point(21000, 1500, input_out_point));

        let hash = blake2b!(input_out_point_tx_hash, 0u64.to_le_bytes());
        let type_id = Bytes::from(hash.to_vec());

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(type_id.clone()).to_vec();
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
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_type_args(type_id);

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
        let mut hash_1 = info_cell_type_hash(info_type_args(0)).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let input_0 = Inputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .liquidity_sudt_type_hash(*LIQUIDITY_SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_witness(witness_args_input_type(0));

        let input_1 = Inputs::new_pool(SudtCell::new(POOL_CAPACITY, 0))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let liquidity_in_lock_args = LiquidityRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(3))
            .version(1)
            .sudt_min(0)
            .ckb_min(0)
            .info_type_hash(info_cell_type_hash(info_type_args(0)))
            .tips(0)
            .tips_sudt(0)
            .build();
        let input_3 = Inputs::new_liquidity(LiquidityRequestCell::new(SUDT_CAPACITY + 50, 50))
            .custom_lock_args(liquidity_in_lock_args.as_bytes())
            .custom_type_args(liquidity_sudt_type_args());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(50)
                .sudt_reserve(50)
                .total_liquidity(50)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_CAPACITY + 50, 50))
            .custom_lock_args(Bytes::from(hash));
        let output_2 = Outputs::new_matcher(FreeCell::new(150));
        let output_3 =
            Outputs::new_sudt(SudtCell::new(100, 50)).custom_type_args(liquidity_sudt_type_args());

        let (mut context, tx) = build_test_context(vec![input_0, input_1, input_2, input_3], vec![
            output_0, output_1, output_2, output_3,
        ]);
        let tx = context.complete_tx(tx);

        let cycle = context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        println!("cycle used {:?}", cycle);

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
        let mut hash_1 = info_cell_type_hash(info_type_args(0)).to_vec();
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
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_witness(witness_args_input_type(0));
        let input_1 = Inputs::new_pool(SudtCell::new(POOL_CAPACITY + 50, 50))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let liquidity_in_lock_args = LiquidityRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(9999))
            .version(1)
            .sudt_min(0)
            .ckb_min(30)
            .info_type_hash(info_cell_type_hash(info_type_args(0)))
            .tips(0)
            .tips_sudt(0)
            .build();
        let input_3 = Inputs::new_liquidity(LiquidityRequestCell::new(SUDT_CAPACITY + 100, 50))
            .custom_lock_args(liquidity_in_lock_args.as_bytes());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(101)
                .sudt_reserve(100)
                .total_liquidity(201)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_CAPACITY + 101, 100))
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

        let cycle = context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        println!("cycle used {:?}", cycle);

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    mint_liquidity_change_sudt_success,
    {
        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(info_type_args(0)).to_vec();
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
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_witness(witness_args_input_type(0));

        // pool_in.capcity = POOL_BASE_CAPCITY + info_in.ckb_reserve
        // pool_in.amount = info_in.sudt_reserve
        let input_1 = Inputs::new_pool(SudtCell::new(POOL_CAPACITY + 50, 50))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let liquidity_in_lock_args = LiquidityRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(9999))
            .version(1)
            .sudt_min(80)
            .ckb_min(30)
            .info_type_hash(info_cell_type_hash(info_type_args(0)))
            .tips(0)
            .tips_sudt(0)
            .build();
        let input_3 =
            Inputs::new_liquidity(LiquidityRequestCell::new(SUDT_CAPACITY * 2 + 100, 302))
                .custom_lock_args(liquidity_in_lock_args.as_bytes());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(150)
                .sudt_reserve(151)
                .total_liquidity(301)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_CAPACITY + 150, 151))
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

        let cycle = context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        println!("cycle used {:?}", cycle);

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    burn_liquidity_success,
    {
        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(info_type_args(0)).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let input_0 = Inputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .total_liquidity(100)
                .sudt_reserve(100)
                .ckb_reserve(100)
                .liquidity_sudt_type_hash(*LIQUIDITY_SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_witness(witness_args_input_type(0));

        let input_1 = Inputs::new_pool(SudtCell::new(POOL_CAPACITY + 100, 100))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let liquidity_in_lock_args = LiquidityRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(9999))
            .version(1)
            .sudt_min(50)
            .ckb_min(30)
            .info_type_hash(info_cell_type_hash(info_type_args(0)))
            .tips(0)
            .tips_sudt(0)
            .build();
        let input_3 = Inputs::new_liquidity(LiquidityRequestCell::new(SUDT_CAPACITY * 2 + 100, 50))
            .custom_lock_args(liquidity_in_lock_args.as_bytes())
            .custom_type_args(liquidity_sudt_type_args());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(49)
                .sudt_reserve(49)
                .total_liquidity(50)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_CAPACITY + 49, 49))
            .custom_lock_args(Bytes::from(hash));
        let output_2 = Outputs::new_matcher(FreeCell::new(150));
        let output_3 = Outputs::new_sudt(SudtCell::new(SUDT_CAPACITY + 50, 51))
            .custom_lock_args(Bytes::from(9999usize.to_le_bytes().to_vec()));
        let output_4 = Outputs::new_ckb(FreeCell::new(SUDT_CAPACITY + 101))
            .custom_lock_args(user_lock_args(9999));

        let (mut context, tx) = build_test_context(vec![input_0, input_1, input_2, input_3], vec![
            output_0, output_1, output_2, output_3, output_4,
        ]);
        let tx = context.complete_tx(tx);

        // let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        // assert_error_eq!(err, tx_error(, 0));

        let cycle = context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        println!("cycle used {:?}", cycle);

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    ckb_swap_sudt_success,
    {
        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(info_type_args(0)).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let input_0 = Inputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .total_liquidity(100)
                .sudt_reserve(100)
                .ckb_reserve(100)
                .liquidity_sudt_type_hash(*LIQUIDITY_SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_witness(witness_args_input_type(1));

        let input_1 = Inputs::new_pool(SudtCell::new(POOL_CAPACITY + 100, 100))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let swap_lock_args = SwapRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(0))
            .version(1)
            .amount_out_min(35)
            .sudt_type_hash(*SUDT_TYPE_HASH)
            .build();
        let input_3 = Inputs::new_swap(SwapRequestCell::new_ckb(SUDT_CAPACITY + 70))
            .custom_lock_args(swap_lock_args.as_bytes());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(170)
                .sudt_reserve(58)
                .total_liquidity(100)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_CAPACITY + 170, 58))
            .custom_lock_args(Bytes::from(hash));
        let output_2 = Outputs::new_matcher(FreeCell::new(150));
        let output_3 =
            Outputs::new_sudt(SudtCell::new(SUDT_CAPACITY, 42)).custom_lock_args(user_lock_args(0));

        let (mut context, tx) = build_test_context(vec![input_0, input_1, input_2, input_3], vec![
            output_0, output_1, output_2, output_3,
        ]);
        let tx = context.complete_tx(tx);

        // let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        // assert_error_eq!(err, tx_error(, 0));

        let cycle = context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        println!("cycle used {:?}", cycle);

        (context, tx)
    },
    false,
    "info-typescript-sim"
);

test_contract!(
    sudt_swap_ckb_success,
    {
        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(info_type_args(0)).to_vec();
        hash.append(&mut hash_1);
        assert_eq!(hash.len(), 64);

        let input_0 = Inputs::new_info(
            InfoCellBuilder::default()
                .capacity(1000)
                .total_liquidity(100)
                .sudt_reserve(100)
                .ckb_reserve(100)
                .liquidity_sudt_type_hash(*LIQUIDITY_SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()))
        .custom_witness(witness_args_input_type(1));

        let input_1 = Inputs::new_pool(SudtCell::new(POOL_CAPACITY + 100, 100))
            .custom_lock_args(Bytes::from(hash.clone()));
        let input_2 = Inputs::new_matcher(FreeCell::new(100));

        let swap_lock_args = SwapRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(0))
            .version(1)
            .amount_out_min(33)
            .sudt_type_hash(*SUDT_TYPE_HASH)
            .build();
        let input_3 = Inputs::new_swap(SwapRequestCell::new_sudt(SUDT_CAPACITY, 50))
            .custom_lock_args(swap_lock_args.as_bytes());

        let output_0 = Outputs::new_info(
            InfoCellBuilder::default()
                .capacity(INFO_CAPACITY)
                .ckb_reserve(66)
                .sudt_reserve(150)
                .total_liquidity(100)
                .liquidity_sudt_type_hash(*SUDT_TYPE_HASH)
                .build(),
        )
        .custom_lock_args(Bytes::from(hash.clone()));
        let output_1 = Outputs::new_pool(SudtCell::new(POOL_CAPACITY + 66, 150))
            .custom_lock_args(Bytes::from(hash));
        let output_2 = Outputs::new_matcher(FreeCell::new(150));
        let output_3 =
            Outputs::new_ckb(FreeCell::new(SUDT_CAPACITY + 34)).custom_lock_args(user_lock_args(0));

        let (mut context, tx) = build_test_context(vec![input_0, input_1, input_2, input_3], vec![
            output_0, output_1, output_2, output_3,
        ]);
        let tx = context.complete_tx(tx);

        // let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
        // assert_error_eq!(err, tx_error(, 0));

        let cycle = context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        println!("cycle used {:?}", cycle);

        (context, tx)
    },
    false,
    "info-typescript-sim"
);
