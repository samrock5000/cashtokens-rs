// Copyright (c) 2023 The Bitcoin developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.


use crate::{
    error::DataError,
    script::Script,
    ser::{BitcoinSer, BitcoinSerializer,},
    tx::{TxId,CashToken,  WrappedTokenScript},
};


/// CTransaction, a Bitcoin transaction.
///
/// ```
/// # use bitcoinsuite_core::tx::{Tx, TxId, Transaction};
/// let txid = TxId::from([3; 32]);
/// let tx = Tx::with_txid(
///     txid,
///     Transaction {
///         version: 1,
///         inputs: vec![],
///         outputs: vec![],
///         locktime: 0,
///     },
/// );
/// assert_eq!(tx.txid(), txid);
/// assert_eq!(tx.version, 1);
/// assert_eq!(tx.inputs, vec![]);
/// assert_eq!(tx.outputs, vec![]);
/// assert_eq!(tx.locktime, 0);
/// ```
///
/// Immutable version of [`Transaction`], so this will fail:
/// ```compile_fail
/// # use bitcoinsuite_core::tx::Tx;
/// let mut tx = Tx::default();
/// tx.version = 1;
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tx {
    txid: TxId,
    tx: Transaction,
}

/// Bitcoin transaction. Mutable version of [`Tx`], like CMutableTransaction.
///
/// The biggest difference is that it doesn't have a txid() method, which we
/// cannot know without hashing the tx every time, which would be expensive to
/// compute.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Transaction {
    /// nVersion of the tx.
    pub version: i32,
    /// Tx inputs.
    pub inputs: Vec<Input>,
    /// Tx outputs.
    pub outputs: Vec<Output>,
    /// Locktime of the tx.
    pub locktime: u32,
}

/// COutPoint, pointing to a coin being spent.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OutPoint {
    /// TxId of the output of the coin.
    pub txid: TxId,
    /// Index in the outputs of the tx of the coin.
    pub outpoint_index: u32,
}

/// Points to an input spending a coin.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpentBy {
    /// TxId of the tx with the input.
    pub txid: TxId,
    /// Index in the inputs of the tx.
    pub input_idx: u32,
}

/// CTxIn, spending an unspent output.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Input {
    /// Points to an output being spent.
    pub prev_out: OutPoint,
    /// scriptSig unlocking the output.
    pub script: Script,
    /// nSequence.
    pub sequence: u32,
    //// Coin being spent by this tx.
   //// May be [`None`] for coinbase txs or if the spent coin is unknown.
    // pub coin: Option<Coin>,
}

/// CTxOut, creating a new output.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Output {
    /// Value of the output.
    pub value: u64,
    /// Script locking the output.
    pub script: Script,
    /// Token output, optional.
    pub token: Option<CashToken>,
}



/// Coin, can be spent by providing a valid unlocking script.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coin {
    /// Output, locking the coins.
    pub output: Output,
    /// Height of the coin in the chain.
    pub height: i32,
    /// Whether the coin is a coinbase.
    pub is_coinbase: bool,
}

impl Tx {
    /// Create a new [`Tx`] with a given [`TxId`] and [`Transaction`].
    ///
    /// It is the responsibility of the caller to ensure the `txid` matches
    /// `tx`.
    pub fn with_txid(txid: TxId, tx: Transaction) -> Self {
        Tx { txid, tx }
    }
    /// [`TxId`] of this [`Tx`].
    pub fn txid(&self) -> TxId {
        self.txid
    }
    /// Like [`Tx::txid`], but as a reference.
    pub fn txid_ref(&self) -> &TxId {
        &self.txid
    }
}

impl std::ops::Deref for Tx {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl BitcoinSer for Transaction {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        self.version.ser_to(bytes);
        self.inputs.ser_to(bytes);
        self.outputs.ser_to(bytes);
        self.locktime.ser_to(bytes);
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        Ok(Transaction {
            version: BitcoinSer::deser(data)?,
            inputs: BitcoinSer::deser(data)?,
            outputs: BitcoinSer::deser(data)?,
            locktime: BitcoinSer::deser(data)?,
        })
    }
}

impl BitcoinSer for Tx {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        Transaction::ser_to(self, bytes)
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        let tx = Transaction::deser(data)?;
        Ok(Tx::with_txid(TxId::from_tx(&tx), tx))
    }
}

impl BitcoinSer for OutPoint {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        self.txid.ser_to(bytes);
        self.outpoint_index.ser_to(bytes);
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        Ok(OutPoint {
            txid: BitcoinSer::deser(data)?,
            outpoint_index: BitcoinSer::deser(data)?,
        })
    }
}

impl BitcoinSer for Input {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        self.prev_out.ser_to(bytes);
        self.script.ser_to(bytes);
        self.sequence.ser_to(bytes);
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        Ok(Input {
            prev_out: BitcoinSer::deser(data)?,
            script: BitcoinSer::deser(data)?,
            sequence: BitcoinSer::deser(data)?,
            // coin: None,
        })
    }
}


impl BitcoinSer for Output {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        self.value.ser_to(bytes);
        if self.token.is_some() {
           vec![ self.token.ser().to_vec(), self.script.to_vec()].concat().ser_to(bytes);
        } else {
            self.script.ser_to(bytes);
        }
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {

        let token_prefix_index = 9;
        let prefix_token = data.slice(token_prefix_index..token_prefix_index + 1);
     
          if prefix_token[0] == 0xef {
            let value:u64 = BitcoinSer::deser(data)?;
            let  WrappedTokenScript(token, script) = Script::unwrap_prefixed_lockscript(data)?;
         
           Ok(Output { value, script, token:Some(token.unwrap()) })
           
        } else {
            let value:u64 = BitcoinSer::deser(data)?;
            let script:Script = BitcoinSer::deser(data)?; 
            let output = Output {
                value,
                script,
                token:None
            };
            Ok(output)
        }
     
        }
}
    

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use crate::hash::{Hashed, Sha256d};
    use crate::ser::CompactUint;
    use crate::{
        script::Script,
        ser::BitcoinSer,
        tx::{OutPoint,token::*, TxId, Input, Transaction, Output},
    };
    use bytes::Bytes;
    fn verify_ser(tx: Transaction, ser: &[u8]) {
        assert_eq!(tx.ser().as_ref(), ser);
        assert_eq!(tx.ser_len(), ser.len());
        let mut bytes = Bytes::copy_from_slice(ser);
        assert_eq!(tx, Transaction::deser(&mut bytes).unwrap());

    }
  
    #[test]
    fn test_ser_tx() -> Result<(), hex::FromHexError> {
           verify_ser(Transaction::default(), &[0; 10]);
        verify_ser(
            Transaction {
                version: 0x12345678,
                inputs: vec![],
                outputs: vec![],
                locktime: 0x9abcdef1,
            },
            &hex::decode("785634120000f1debc9a")?,
        );
        let genesis_tx = Transaction {
            version: 1,
            inputs: vec![Input {
                prev_out: OutPoint {
                    txid: TxId::from([0; 32]),
                    outpoint_index: 0xffff_ffff,
                },
                script: Script::new(
                    hex::decode(
                        "04ffff001d0104455468652054696d65732030332f4a616e2f3230\
                         3039204368616e63656c6c6f72206f6e206272696e6b206f662073\
                         65636f6e64206261696c6f757420666f722062616e6b73",
                    )?
                    .into(),
                ),
                sequence: 0xffff_ffff,
              
            }],
            outputs: vec![Output {
                value: 5000000000,
                script: Script::new(
                    hex::decode(
                        "4104678afdb0fe5548271967f1a67130b7105cd6a828e03909a679\
                         62e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7\
                         ba0b8d578a4c702b6bf11d5fac",
                    )?
                    .into(),
                ),
                token: None,
            }],
            locktime: 0,
        };
        verify_ser(
            genesis_tx,
            &hex::decode(
                "01000000010000000000000000000000000000000000000000000000000000\
                 000000000000ffffffff4d04ffff001d0104455468652054696d6573203033\
                 2f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f\
                 66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff01\
                 00f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828\
                 e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384d\
                 f7ba0b8d578a4c702b6bf11d5fac00000000",
            )?,
        );
        let txid1 = "73db6c3702cbe1b1f918a2b68fb1b4c6b274520c385c62c8a8f01ea5388ffaf5";
        let txid2 = "f712ca870ce69766bb6769c0051e819514da5526d2fd4853c3c52f27bb4af450";
       
        let category_id = Sha256d::from_be_hex("56baff1b1ef705961d733ed985c7740d9035df03e830aa4564afa62a0f931165").unwrap();

        let single_fungible_token_out = Transaction {
            version: 2,
            inputs: vec![Input {
                prev_out: OutPoint {
                    txid: TxId::from(Sha256d::from_be_hex(txid1).unwrap() ),
                    outpoint_index: 0,
                },
                script: Script::new(
                    hex::decode(
                        "418fb787523e8950b4f0329784483824574fd3e775d6430622e10b51c2b616c33e68de26ab08b728a50a08585946d3decbf4e9d8e1265154245db744ab9cd084e94121032d3d167556d62e3376c54db8e98d52f06b7e4f0e02c153b7a62333847a604d37",
                    )?
                    .into(),
                ),
                sequence: 0,
               
            },
            Input {
                prev_out: OutPoint {
                    txid: TxId::from(Sha256d::from_be_hex(txid2).unwrap() ),
                    outpoint_index: 0,
                },
                script: Script::new(
                    hex::decode(
                        "2400ce206511930f2aa6af6445aa30e803df35900d74c785d93e731d9605f71e1bffba5687",
                    )?
                    .into(),
                ),
                sequence: 0,
               
            },
            
            ],
            outputs: vec![Output {
                value: 8700,
                script: Script::new(
                    hex::decode(
                        "76a91457314787eafac80afd059f1f31e990d7db9b70fd88ac",
                    )?
                    .into(),
                ),
                token: Some( CashToken{
                    amount:CompactUint(1),
                    category: TxId::from(category_id),
                    nft: None
            })
            }],
            locktime: 0,
        };

        let tx_hex: Vec<u8> = hex::decode(
            "02000000\
            02\
            f5fa8f38a51ef0a8c8625c380c5274b2c6b4b18fb6a218f9b1e1cb02376cdb73\
            00000000\
            64\
            41\
            8fb787523e8950b4f0329784483824574fd3e775d6430622e10b51c2b616c33e68de26ab08b728a50a08585946d3decbf4e9d8e1265154245db744ab9cd084e94121032d3d167556d62e3376c54db8e98d52f06b7e4f0e02c153b7a62333847a604d37\
            00000000\
            50f44abb272fc5c35348fdd22655da1495811e05c06967bb6697e60c87ca12f7\
            00000000\
            25\
            2400ce206511930f2aa6af6445aa30e803df35900d74c785d93e731d9605f71e1bffba5687\
            00000000\
            01\
            fc21000000000000\
            3c\
            ef\
            6511930f2aa6af6445aa30e803df35900d74c785d93e731d9605f71e1bffba56\
            10\
            01\
            76a91457314787eafac80afd059f1f31e990d7db9b70fd88ac\
            00000000",
        ).unwrap();

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
                single_fungible_token_out,
                &tx_hex,
        );
        verify_ser(
            multiple_token_out,
            &multi_out_hex
        );
        Ok(())
    }

    
}   