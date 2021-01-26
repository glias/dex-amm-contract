use ckb_tool::ckb_jsonrpc_types::Script;
use ckb_tool::ckb_types::prelude::Unpack;

use super::*;

test_contract!(
    info_creation,
    {
        let input = Inputs::new_sudt(SudtCell::new(21000, 1500));

        let mut hash = blake2b!("ckb", *SUDT_TYPE_HASH).to_vec();
        let mut hash_1 = info_cell_type_hash(Bytes::from(0usize.to_le_bytes().to_vec())).to_vec();
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
