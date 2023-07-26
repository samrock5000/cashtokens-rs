# De/Serialize Transaction with Token Data

```rust
use use bitcoinsuite_core::tx::*;

        let vin0_txid = "45d926f0ddc969e5bd2976b0eac90320eed5850fb27928718cbcd2f5936b52f5";
        let vin1_txid = "8561fb07bf0656b37a5d627f33b3153edfde42e5817a9e34e91b6a2bdd4307ae";
        let vin2_txid = "8561fb07bf0656b37a5d627f33b3153edfde42e5817a9e34e91b6a2bdd4307ae";
        let vin3_txid = "148efbc10c51abeed89640ceb1022246a7d2ed400a29909512a7919bad825efd";

        let token_hash2 = Sha256d::from_be_hex("f35a1d9889fd05d8880954ff1b3525acd97f2b20fa4247b9444a197fdfe9a806").unwrap();
        let token_hash3 = Sha256d::from_be_hex("d7ef16e26e8b95dbe7f89ed7809b68b293b39eb907e052c317a9d7f1fcf4a98c").unwrap();

        let multi_out_hex = hex::decode("0200000004f5526b93f5d2bc8c712879b20f85d5ee2003c9eab07629bde569c9ddf026d94500000000900440eb41064c895600a06951ce7804000000409f69c0009d00cd00c78800d200cf8800d18851cf81009d52d078527902f40196937804000000409678957b0400000040977b9504ffffff3f930400000040969352d39d52c6787b02f40196946e9604000000409552795279970400000040955279938c7b969352cc789d7c9451c651cc7c7b02f40196939d51d351d09c00000000ae0743dd2b6a1be9349e7a81e542dedf3e15b3337f625d7ab35606bf07fb6185010000002b2a5600a069c0519d51ce51cd51c78851d251cf8851d1788800ce8800cf815178a17c55a19a6951c852c88700000000ae0743dd2b6a1be9349e7a81e542dedf3e15b3337f625d7ab35606bf07fb61850200000017165600a069c0529d52cd52c78852d152ce8851c852c88700000000fd5e82ad9b91a7129590290a40edd2a7462202b1ce4096d8eeab510cc1fb8e140100000064410b856f91dc38127002749a5ea0c42f51ceed2ae16f7fe7e49e25a66da265b130badd2c0126e747a58b67e07dbb47e3b17ea2710d440502ef27d677bcdd5387c2412103ad1b7b97f1b5ab0c37294a3f4c7e5bd6e4350ae43b7a2011dcfbfcec1592e27a0000000005e8030000000000003bef06a8e9df7f194a44b94742fa202b7fd9ac25351bff540988d805fd89981d5af3600103a914768bdbe93350a5809269deb06fb62a0f931ff5d1873d9700000000000044ef06a8e9df7f194a44b94742fa202b7fd9ac25351bff540988d805fd89981d5af3700100ffc362a9faffff0f00a9143d028e942455c9447dfd526d87d035c117aea8ab874816a900000000003eef8ca9f4fcf1d7a917c352e007b99eb393b2689b80d79ef8e7db958b6ee216efd710fe2eab4230a914320616788844eff2f1fbb3068d3dd572a6bddae687e80300000000000044ef8ca9f4fcf1d7a917c352e007b99eb393b2689b80d79ef8e7db958b6ee216efd710ffc85f59bc0100000076a9145ac444947356a03b6aa8a096ca7b48d985b683fb88ac83f03a00000000001976a9145ac444947356a03b6aa8a096ca7b48d985b683fb88ac00000000").unwrap();

        // txid ff68509e4dd9e2f5f3f41a2f1a7866b72a7b3600ca3ddaf88ba02b9263833f1d
        // https://chipnet.chaingraph.cash/tx/ff68509e4dd9e2f5f3f41a2f1a7866b72a7b3600ca3ddaf88ba02b9263833f1d

        let multiple_token_out = Transaction {
            version: 2,
            inputs: vec![
                 //vin 0
                 Input {
                prev_out: OutPoint {
                    txid: TxId::from(Sha256d::from_be_hex(vin0_txid).unwrap() ),
                    outpoint_index: 0,
                },
                script: Script::new(
                    hex::decode(
                        "0440eb41064c895600a06951ce7804000000409f69c0009d00cd00c78800d200cf8800d18851cf81009d52d078527902f40196937804000000409678957b0400000040977b9504ffffff3f930400000040969352d39d52c6787b02f40196946e9604000000409552795279970400000040955279938c7b969352cc789d7c9451c651cc7c7b02f40196939d51d351d09c",
                    )?
                    .into(),
                ),
                sequence: 0,

            },
            //vin 1
            Input {
                prev_out: OutPoint {
                    txid: TxId::from(Sha256d::from_be_hex(vin1_txid).unwrap() ),
                    outpoint_index: 1,
                },
                script: Script::new(
                    hex::decode(
                        "2a5600a069c0519d51ce51cd51c78851d251cf8851d1788800ce8800cf815178a17c55a19a6951c852c887",
                    )?
                    .into(),
                ),
                sequence: 0,

            },
            //vin 2
            Input {
                prev_out: OutPoint {
                    txid: TxId::from(Sha256d::from_be_hex(vin2_txid).unwrap() ),
                    outpoint_index: 2,
                },
                script: Script::new(
                    hex::decode(
                        "165600a069c0529d52cd52c78852d152ce8851c852c887",
                    )?
                    .into(),
                ),
                sequence: 0,

            },
              //vin 3
              Input {
                prev_out: OutPoint {
                    txid: TxId::from(Sha256d::from_be_hex(vin3_txid).unwrap() ),
                    outpoint_index: 1,
                },
                script: Script::new(
                    hex::decode(
                        "410b856f91dc38127002749a5ea0c42f51ceed2ae16f7fe7e49e25a66da265b130badd2c0126e747a58b67e07dbb47e3b17ea2710d440502ef27d677bcdd5387c2412103ad1b7b97f1b5ab0c37294a3f4c7e5bd6e4350ae43b7a2011dcfbfcec1592e27a",
                    )?
                    .into(),
                ),
                sequence: 0,

            }

            ],
            outputs: vec![
                //INDEX 0
            Output {
                value: 1000,
                script: Script::new(
                    hex::decode(
                        "a914768bdbe93350a5809269deb06fb62a0f931ff5d187",
                    )?
                    .into(),
                ),
                token: Some( CashToken{
                    amount:CompactUint(0),
                    category: TxId::from(token_hash2),
                    nft: Some(NFT {
                        capability:NonFungibleTokenCapability(Capability::None),
                        commitment:Commitment( hex::decode("03")?.into()

                                )
                            }
                        )
                     })
            },
            //INDEX 1
            Output {
                value: 38717,
                script: Script::new(
                    hex::decode(
                        "a9143d028e942455c9447dfd526d87d035c117aea8ab87",
                    )?
                    .into(),
                ),
                token: Some( CashToken{
                    amount:CompactUint(4503599537808067),
                    category: TxId::from(token_hash2),
                    nft: Some(NFT {
                        capability:NonFungibleTokenCapability(Capability::None),
                        commitment:Commitment( hex::decode("00")?.into()

                                )
                            }
                        )
                     })
            },
              //INDEX 2
            Output {
                value: 11081288,
                script: Script::new(
                    hex::decode(
                        "a914320616788844eff2f1fbb3068d3dd572a6bddae687",
                    )?
                    .into(),
                ),
                token: Some( CashToken{
                    amount:CompactUint(809675566),
                    category: TxId::from(token_hash3),
                    nft: None
                     })
            },
            //INDEX 3
            Output {
                value: 1000,
                script: Script::new(
                    hex::decode(
                        "76a9145ac444947356a03b6aa8a096ca7b48d985b683fb88ac",
                    )?
                    .into(),
                ),
                token: Some( CashToken{
                    amount:CompactUint(7454941128),
                    category: TxId::from(token_hash3),
                    nft: None
                     })
            },
             //INDEX 4
            Output {
                value: 3862659,
                script: Script::new(
                    hex::decode(
                        "76a9145ac444947356a03b6aa8a096ca7b48d985b683fb88ac",
                    )?
                    .into(),
                ),
                token: None
            },
            ],
            locktime: 0,
        };

        verify_ser(
            multiple_token_out,
            &multi_out_hex
        );
        Ok(())

    fn verify_ser(tx: Transaction, ser: &[u8]) {
        assert_eq!(tx.ser().as_ref(), ser);
        assert_eq!(tx.ser_len(), ser.len());
        let mut bytes = Bytes::copy_from_slice(ser);
        assert_eq!(tx, Transaction::deser(&mut bytes).unwrap());

    }


```

# Example

get token data from raw tx hex:

```bash
cargo build -p token-cli --release

./target/release/token-cli txtokens 0200000004f5526b93f5d2bc8c712879b20f85d5ee2003c9eab07629bde569c9ddf026d94500000000900440eb41064c895600a06951ce7804000000409f69c0009d00cd00c78800d200cf8800d18851cf81009d52d078527902f40196937804000000409678957b0400000040977b9504ffffff3f930400000040969352d39d52c6787b02f40196946e9604000000409552795279970400000040955279938c7b969352cc789d7c9451c651cc7c7b02f40196939d51d351d09c00000000ae0743dd2b6a1be9349e7a81e542dedf3e15b3337f625d7ab35606bf07fb6185010000002b2a5600a069c0519d51ce51cd51c78851d251cf8851d1788800ce8800cf815178a17c55a19a6951c852c88700000000ae0743dd2b6a1be9349e7a81e542dedf3e15b3337f625d7ab35606bf07fb61850200000017165600a069c0529d52cd52c78852d152ce8851c852c88700000000fd5e82ad9b91a7129590290a40edd2a7462202b1ce4096d8eeab510cc1fb8e140100000064410b856f91dc38127002749a5ea0c42f51ceed2ae16f7fe7e49e25a66da265b130badd2c0126e747a58b67e07dbb47e3b17ea2710d440502ef27d677bcdd5387c2412103ad1b7b97f1b5ab0c37294a3f4c7e5bd6e4350ae43b7a2011dcfbfcec1592e27a0000000005e8030000000000003bef06a8e9df7f194a44b94742fa202b7fd9ac25351bff540988d805fd89981d5af3600103a914768bdbe93350a5809269deb06fb62a0f931ff5d1873d9700000000000044ef06a8e9df7f194a44b94742fa202b7fd9ac25351bff540988d805fd89981d5af3700100ffc362a9faffff0f00a9143d028e942455c9447dfd526d87d035c117aea8ab874816a900000000003eef8ca9f4fcf1d7a917c352e007b99eb393b2689b80d79ef8e7db958b6ee216efd710fe2eab4230a914320616788844eff2f1fbb3068d3dd572a6bddae687e80300000000000044ef8ca9f4fcf1d7a917c352e007b99eb393b2689b80d79ef8e7db958b6ee216efd710ffc85f59bc0100000076a9145ac444947356a03b6aa8a096ca7b48d985b683fb88ac83f03a00000000001976a9145ac444947356a03b6aa8a096ca7b48d985b683fb88ac00000000

```
