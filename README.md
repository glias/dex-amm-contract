# dex-amm-contract

The Glias DEX AMM contract with ckb and SUDT pair.

## Pre-requirment

* [capsule](https://github.com/nervosnetwork/capsule) >= 0.4.4
* [ckb-cli](https://github.com/nervosnetwork/ckb-cli) >= 0.39.0

> Note: Capsule uses [docker](https://docs.docker.com/get-docker/) to build contracts and run tests. Besides, docker and ckb-cli must be accessible in the PATH in order for them to be used by Capsule.

## Getting Start

* Build contract

```shell
capsule build --release
```

* Run tests

```shell
make schema
make test
```

## Transaction View

### Create Pool

```
                                info_cell
any_token_cell    ------->
                                pool_cell
```

### Initial Mint Liquidity

```
info_in_cell                            info_out_cell
pool_in_cell                            pool_out_cell
                          ------->
matcher_in_cell                    matcher_out_cell 
add_liquidity_cell                      liquidity_sudt_cell
```

### Swap And Liquidity Transaction

```
info_in_cell                            info_out_cell
pool_in_cell                            pool_out_cell
                          ------->
matcher_in_cell                    matcher_out_cell 
[swap_request_cell]                     [sudt_cell or ckb_cell]

[removed_liquidity_cell]                [sudt_cell
                                        + ckb_cell]
                                        
[add_liquidity_cell]                    [liquidity_cell
                                        + (sudt_cell or ckb_cell)]
```

> Notice that the witness argument of index zero in inputs should contain the count of swap request cell. The count should be encoded into a little-endian byte array and saved in the `input_type` field, except create pool transaction.

##  Deployment

### 1. Update the deployment configurations

Open `deployment.toml` :

- cells describes which cells to be deployed.

  - `name`: Define the reference name used in the deployment configuration.
  - `enable_type_id` : If it is set to true means create a type_id for the cell.
  - `location` : Define the script binary path.
  - `dep_groups` describes which dep_groups to be created. Dep Group is a cell which bundles several cells as its members. When a dep group cell is used in cell_deps, it has the same effect as adding all its members into cell_deps. In our case, we don’t need dep_groups.

- `lock` describes the lock field of the new deployed cells.It is recommended to set lock to the address(an address that you can unlock) of deployer in the dev chain and in the testnet, which is easier to update the script.

### 2. Build release version of the script

The release version of script doesn’t include debug symbols which makes the size smaller.

```shell
capsule build --release
```

#### 3. Deploy the script

```shell
capsule deploy --address <ckt1....> --fee 0.001
```

If the `ckb-cli` has been installed and `dev-chain` RPC is connectable, you will see the deployment plan:

new_occupied_capacity and total_occupied_capacity refer how much CKB to store cells and data.
txs_fee_capacity refers how much CKB to pay the transaction fee.

### 4. Type yes or y and input the password to unlock the account.

```shell
send cell_tx 0xcdfd397823f6a130294c72fbe397c469d459b83db401296c291db7b170b15839
Deployment complete
```

Now the dex script has been deployed, you can refer to this script by using `tx_hash: 0xcdfd397823f6a130294c72fbe397c469d459b83db401296c291db7b170b15839 index: 0` as `out_point`(your tx_hash should be another value).
