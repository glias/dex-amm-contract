use super::*;

test_contract!(
    liquidity_request_cancel_success,
    {
        let input_0 =
            Inputs::new_matcher(FreeCell::new(150)).custom_witness(witness_args_input_type(0));

        let liquidity_in_lock_args = LiquidityRequestLockArgsBuilder::default()
            .user_lock_hash(user_lock_hash(0))
            .version(1)
            .sudt_min(80)
            .ckb_min(30)
            .info_type_hash(info_cell_type_hash(info_type_args(0)))
            .tips(0)
            .tips_sudt(0)
            .build();
        let input_1 =
            Inputs::new_liquidity(LiquidityRequestCell::new(SUDT_CAPACITY * 2 + 100, 302))
                .custom_lock_args(liquidity_in_lock_args.as_bytes());

        let output_0 = Outputs::new_matcher(FreeCell::new(150));

        let (mut context, tx) = build_test_context(vec![input_0, input_1], vec![output_0]);
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
