// Copyright (c) 2023 The Bitcoin developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use std::ops::BitAnd;

use bytes::Bytes;

use crate::{
    error::DataError,
    script::Script,
    ser::{BitcoinSer, BitcoinSerializer, CompactUint, read_compact_uint_minimal},
    tx::TxId, bytes::read_bytes,
};

/// PREFIX_TOKEN is defined at codepoint 0xef (239) and indicates the presence of a token prefix
const TOKEN_PREFIX: u8 = 0xef;

/// Maximum fungible token amount
const MAXIMUM_TOKEN_AMOUNT: u64 = 9223372036854775807;

/// The minimum possible length is 34
pub const MINIMUM_PREFIX_LENGTH: u8 = 34;
 
/// Single fungible [CashToken] deser example output:
/// ```
/// 
/// # use bytes::Bytes;
/// # use bitcoinsuite_core::tx::{Transaction,Input,OutPoint,TxId,Output,CashToken};
/// # use bitcoinsuite_core::hash::Sha256d;
/// # use bitcoinsuite_core::script::Script;
/// # use bitcoinsuite_core::ser::{CompactUint,BitcoinSer};
/// # use crate::bitcoinsuite_core::hash::Hashed;
/// 
/// let tx_hex: Vec<u8> = hex::decode(
///     "02000000\
///     02\
///     f5fa8f38a51ef0a8c8625c380c5274b2c6b4b18fb6a218f9b1e1cb02376cdb73\
///     00000000\
///     64\
///     41\
///     8fb787523e8950b4f0329784483824574fd3e775d6430622e10b51c2b616c33e68de26ab08b728a50a08585946d3decbf4e9d8e1265154245db744ab9cd084e94121032d3d167556d62e3376c54db8e98d52f06b7e4f0e02c153b7a62333847a604d37\
///     00000000\
///     50f44abb272fc5c35348fdd22655da1495811e05c06967bb6697e60c87ca12f7\
///     00000000\
///     25\
///     2400ce206511930f2aa6af6445aa30e803df35900d74c785d93e731d9605f71e1bffba5687\
///     00000000\
///     01\
///     fc21000000000000\
///     3c\
///     ef\
///     6511930f2aa6af6445aa30e803df35900d74c785d93e731d9605f71e1bffba56\
///     10\
///     01\
///     76a91457314787eafac80afd059f1f31e990d7db9b70fd88ac\
///     00000000",
/// ).unwrap(); 
/// 
/// let txid1 = "73db6c3702cbe1b1f918a2b68fb1b4c6b274520c385c62c8a8f01ea5388ffaf5";
/// let txid2 = "f712ca870ce69766bb6769c0051e819514da5526d2fd4853c3c52f27bb4af450";
///    
/// let category_id = Sha256d::from_be_hex("56baff1b1ef705961d733ed985c7740d9035df03e830aa4564afa62a0f931165").unwrap();
///
/// let single_fungible_token_out = Transaction {
///     version: 2,
///     inputs: vec![Input {
///         prev_out: OutPoint {
///             txid: TxId::from(Sha256d::from_be_hex(txid1).unwrap() ),
///             outpoint_index: 0,
///         },
///         script: Script::new(
///             hex::decode(
///                 "418fb787523e8950b4f0329784483824574fd3e775d6430622e10b51c2b616c33e68de26ab08b728a50a08585946d3decbf4e9d8e1265154245db744ab9cd084e94121032d3d167556d62e3376c54db8e98d52f06b7e4f0e02c153b7a62333847a604d37",
///             ).unwrap()
///             .into(),
///         ),
///         sequence: 0,
///        
///     },
///     Input {
///         prev_out: OutPoint {
///             txid: TxId::from(Sha256d::from_be_hex(txid2).unwrap() ),
///             outpoint_index: 0,
///         },
///         script: Script::new(
///             hex::decode(
///                 "2400ce206511930f2aa6af6445aa30e803df35900d74c785d93e731d9605f71e1bffba5687",
///             ).unwrap()
///             .into(),
///         ),
///         sequence: 0,
///        
///     },
///     
///     ],
///     outputs: vec![Output {
///         value: 8700,
///         script: Script::new(
///             hex::decode(
///                 "76a91457314787eafac80afd059f1f31e990d7db9b70fd88ac",
///             ).unwrap()
///             .into(),
///         ),
///         token: Some( CashToken{
///             amount:CompactUint(1),
///             category: TxId::from(category_id),
///             nft: None
///     })
///     }],
///     locktime: 0,
/// };
/// assert_eq!(single_fungible_token_out ,Transaction::deser(&mut Bytes::copy_from_slice(&tx_hex)).unwrap() )
/// ```

/// The CashToken contents of an [`Output`]. This property is only defined if the
/// output contains one or more tokens. For details, see
/// <https://cashtokens.org/docs/spec/chip#transaction-output-data-model>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CashToken {
    /// The number of fungible tokens held in this output (an integer between 1 and 9223372036854775807).
    /// can be 0 only if NFT is present.
    pub amount: CompactUint,
    ///	The 32-byte ID [`TxId`]of the token category to which the token(s) in this output belong.
    pub category: TxId,
    /// Optional nft field.
    pub nft: Option<NFT>,
    
}

impl CashToken {
    /// Return the [`Commitment`] of this [`CashToken`] if length is 0 returns empty vec.
    pub fn commitment(&self) -> Commitment {
        if self.nft.as_ref().is_none() {
            Commitment(Bytes::from([].as_ref()))
        } else {
            self.nft.to_owned().unwrap().commitment
        }
    }

    /// Token Data encoding
    fn encode(&self) -> Result<TokenData, DataError> {
        let token_data = TokenData {
            prefix: TOKEN_PREFIX,
            category: self.category,
            bitfield: TokenBitfield(self.bitfield()),
            commitment: self.commitment(),
            amount: self.amount,
           
        };
        Ok(token_data)
    }
}

/// Invalidates incorrect token format and capability token data.
pub fn validate_prefix_format_and_capability(data: &mut Bytes) -> Result<(), DataError> {
    let prefix_len = data.len() as usize;
    if prefix_len < MINIMUM_PREFIX_LENGTH as usize {
        return Err(DataError::InvalidTokenPrefixLength {
            minimum_length: MINIMUM_PREFIX_LENGTH as usize,
            actual: prefix_len as usize,
        });
    }
    let token_bitfield_index = 33;
    let bitfield = data.slice(token_bitfield_index..34);
    let bitfield = TokenBitfield(bitfield[0]);
    let prefix_structure = bitfield.0 & TokenBitFlags::TokenFormatMask as u8;
    let nft_capapability_bit = bitfield.0 & TokenBitFlags::NftCapabilityMask as u8;

    if prefix_structure & TokenBitFlags::ReservedBit as u8 != 0 {
        return Err(DataError::InvalidPrefixEncoding {
            error: String::from("Invalid token prefix: reserved bit is set"),
        });
    }
    if nft_capapability_bit > 2 {
        return Err(DataError::InvalidCapability {
            expected:
                "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                    .to_string(),
            actual: nft_capapability_bit,
        });
    }
    if bitfield.has_commitment_length() && !bitfield.has_nft() {
        return Err(DataError::CommitmentWithoutNft {
            error: "Invalid token prefix: commitment requires an NFT".to_string(),
        });
    }

    if bitfield.has_nft() {
        if bitfield.has_commitment_length()
            && data.len() == MINIMUM_PREFIX_LENGTH as usize
        {
            return Err(DataError::CommitmentLengthZero {
                error: "Error reading CompactUint: requires at least one byte".to_owned(),
            });
        }
    }
    if !bitfield.has_nft() && !bitfield.has_amount() {
        return Err(DataError::NoTokens {
            error: "Invalid token prefix: must encode at least one token".to_string(),
        });
    }
    Ok(())
}


impl BitcoinSer for Option<CashToken> {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        match self {
            Some(token) => {
                let tokendata = token.encode().unwrap();
                tokendata.prefix.ser_to(bytes);
                tokendata.category.ser_to(bytes);
                tokendata.bitfield.ser_to(bytes);
                if tokendata.has_commitment_length() {
                    tokendata.commitment.ser_to(bytes);
                    tokendata.amount.ser_to(bytes);
                } else {
                    tokendata.amount.ser_to(bytes);
                }
            }
            None => (),
        }
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        let token_data: TokenData = BitcoinSer::deser(data)?;
        Ok(Some(token_data.decode()?))
    }
}

impl BitcoinSer for CashToken {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        let tokendata = self.encode().unwrap();
        tokendata.prefix.ser_to(bytes);
        tokendata.category.ser_to(bytes);
        tokendata.bitfield.ser_to(bytes);
        if tokendata.has_commitment_length() {
            tokendata.commitment.ser_to(bytes);
            tokendata.amount.ser_to(bytes);
        } else {
            tokendata.amount.ser_to(bytes);
        }
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        let token_data: TokenData = BitcoinSer::deser(data)?;
        Ok(token_data.decode()?)
    }
}

/// Token Encoding for [`CashToken`] prefix_structure ([`TokenBitfield`] & 0xf0)
/// see <https://cashtokens.org/docs/spec/chip/#token-encoding>
#[derive(Debug)]
pub enum TokenBitFlags {
    /// RESERVED_BIT, must be unset.
    ReservedBit = 0b10000000,
    /// The prefix encodes a commitment length and commitment.
    HasCommitmentLength = 0b01000000,
    /// The prefix encodes a non-fungible token.
    HasNFT = 0b00100000,
    /// The prefix encodes an amount of fungible tokens.
    HasAmount = 0b00010000,
    /// Indicates the token structure: ([`TokenBitfield`] & [`TokenBitFlags::TokenFormatMask`])
    TokenFormatMask = 0xf0,
    /// Indicates the token capability: ([`TokenBitfield`] & [`TokenBitFlags::NftCapabilityMask`])
    NftCapabilityMask = 0x0f,
    /// Used to convert false values
    UnSet = 0x00,
}

/// The CashToken contents of an [`Output`]. This property is only defined if the
/// output contains one or more tokens. For details, see
/// <https://cashtokens.org/docs/spec/chip#transaction-output-data-model>
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TokenData {
    ///PREFIX_TOKEN is defined at codepoint 0xef (239) and indicates the presence of a token prefix
    pub prefix: u8,
    /// The 32-byte ID of the token category to which the token(s) in this output
    /// belong in big-endian byte order. This is the byte order typically seen in
    /// block explorers and user interfaces (as opposed to little-endian byte
    /// order, which is used in standard P2P network messages).
    pub category: TxId,

    /// Token bitfield byte. High order nibble is one of the Structure enum values and low order nibble is Capability.
    pub bitfield: TokenBitfield,

    /// The number of fungible tokens (of `category`) held in this output.
    /// Because `Number.MAX_SAFE_INTEGER` (`9007199254740991`) is less than the
    /// maximum token amount (`9223372036854775807`), this value is encoded as
    /// a `u64`.
    pub amount: CompactUint,
    /// If present, the non-fungible token (NFT) held by this output.
    pub commitment: Commitment,
}

impl TokenData {
    fn decode_nft(&self) -> Result<NFT, DataError> {
        let commitment = Commitment(self.commitment.0.to_owned());
        let capability: Result<Capability,DataError> = match  self.capability() {
            0x00 => Ok(Capability::None),
            0x01 => Ok(Capability::Mutable),
            0x02 => Ok(Capability::Minting),
            err => Err(DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: err,
            }) 
        };
        Ok(NFT {
            capability: NonFungibleTokenCapability(capability?),
            commitment,
        })
    }
    fn decode(&self) -> Result<CashToken, DataError> {
        let token = if self.has_nft() {
            let nft = Some(TokenData::decode_nft(&self)?);
            Ok(CashToken {
                amount: self.amount,
                category: self.category,
                nft,
            })
        } else {
            Ok(CashToken {
                amount: self.amount,
                category: self.category,
                nft: None,
            })
        };
        token
    }
}

impl BitcoinSer for TokenData {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        self.prefix.ser_to(bytes);
        self.category.ser_to(bytes);
        self.bitfield.ser_to(bytes);
        if self.has_commitment_length() {
            self.commitment.ser_to(bytes);
            if self.has_amount() {
                self.amount.ser_to(bytes);
            }
        }
        self.amount.ser_to(bytes);
    }
    fn deser(data: &mut Bytes) -> Result<Self, DataError> {
        validate_prefix_format_and_capability(data)?;
        let prefix: u8 = BitcoinSer::deser(data)?;
        let category: TxId = BitcoinSer::deser(data)?;
        let bitfield: TokenBitfield = BitcoinSer::deser(data)?;

        if bitfield.has_commitment_length() {
            //Commitment length field
            if  data.slice(0..)[0] == 0 {
                return Err(DataError::CommitmentLengthZero {
                    error:
                        "Invalid token prefix: if encoded, commitment length must be greater than 0"
                            .to_string()
                });
}
            let commitment: Commitment = BitcoinSer::deser(data)?;
            if commitment.0.is_empty() {
                return Err(DataError::CommitmentLengthZero {
                    error:
                        "Invalid token prefix: if encoded, commitment length must be greater than 0"
                            .to_string(),
                });
            }
            if bitfield.has_amount() {
                let amount: CompactUint = BitcoinSer::deser(data)?;
                if amount.0 > MAXIMUM_TOKEN_AMOUNT {
                    return Err(DataError::InvalidAmountEncoding { 
                        error: "Invalid token prefix: exceeds maximum fungible token amount of 9223372036854775807".to_string(),
                 });
                }
                return Ok(TokenData {
                    prefix,
                    category,
                    bitfield,
                    amount,
                    commitment,
                });
            }
            return Ok(TokenData {
                prefix,
                category,
                bitfield,
                amount: CompactUint(0),
                commitment,
            });
        } else if bitfield.has_nft() && !bitfield.has_commitment_length() {
            if bitfield.has_amount() {
                if data.len() == 0 {
                    return Err(DataError::InvalidAmountEncoding {
                        error: "Error reading CompactUint: requires at least one byte".to_string(),
                    });
                };
                if data[0] as usize == 0 as usize {
                    return Err(DataError::InvalidAmountEncoding {
                        error: "fungible token amount must be greater than 0".to_string(),
                    });
                }

                let amount: CompactUint = BitcoinSer::deser(data)?;
                if amount.0 > MAXIMUM_TOKEN_AMOUNT {
                    return Err(DataError::InvalidAmountEncoding { 
                        error: "Invalid token prefix: exceeds maximum fungible token amount of 9223372036854775807".to_string(),
                 });
                }
                return Ok(TokenData {
                    prefix,
                    category,
                    bitfield,
                    amount,
                    commitment:Commitment(Default::default()),
                });
            }
             return  Ok(TokenData {
                prefix,
                category,
                bitfield,
                amount: CompactUint(0),
                commitment:Commitment(Default::default()),
            });
        }
        if bitfield.capability() != Capability::None as u8 {
            return Err(DataError::CapabilityWithoutNft);
        }
        if data.len() == 0 {
            return Err(DataError::InvalidAmountEncoding {
                error: "Error reading CompactUint: requires at least one byte".to_string(),
            });
        };
        if data[0] as usize == 0 as usize {
            return Err(DataError::InvalidAmountEncoding {
                error: "fungible token amount must be greater than 0".to_string(),
            });
        }
        let amount: CompactUint = BitcoinSer::deser(data)?;
        if amount == CompactUint(0) {
            return Err(DataError::InvalidAmountEncoding {
                error: "fungible token amount must be greater than 0".to_string(),
            });
        }
        if amount.0 > MAXIMUM_TOKEN_AMOUNT {
            return Err(DataError::InvalidAmountEncoding { 
                error: "Invalid token prefix: exceeds maximum fungible token amount of 9223372036854775807".to_string(),
         });
        }
   
        return Ok(TokenData {
            prefix,
            category,
            bitfield,
            amount,
            commitment: Commitment(vec![].into()),
        });
    }
}

impl Bitfield for TokenData {
    /// The payload encodes a commitment-length and a commitment (HasNFT must also be set).
    fn has_commitment_length(&self) -> bool {
        self.bitfield.0 & TokenBitFlags::HasCommitmentLength as u8 != 0
    }

    /// The payload encodes an amount of fungible tokens.
    fn has_amount(&self) -> bool {
        self.bitfield.0 & TokenBitFlags::HasAmount as u8 != 0
    }

    ///  Return [`Capability`]
    fn capability(&self) -> u8 {
        /* match */
        self.bitfield.0 & TokenBitFlags::NftCapabilityMask as u8
    }

    ///  see [`TokenBitFlags::HasNFT`].
    fn has_nft(&self) -> bool {
        self.bitfield.0 & TokenBitFlags::HasNFT as u8 != 0
    }
    ///  true if [`TokenBitFlags::ReservedBit`] is set. Should not be set.
    fn reserved_bit_is_set(&self) -> bool {
        self.bitfield.0 & TokenBitFlags::ReservedBit as u8 != 0
    }
    fn bitfield(&self) -> u8 {
        TokenBitfield(self.bitfield.0).bitfield()
    }
}
/// If present, the non-fungible token (NFT) held by an [`Output`]. If the
/// output does not include a non-fungible token, `None`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NFT {
    /// The [Capability] of this non-fungible token.
    pub capability: NonFungibleTokenCapability,
    /// The commitment contents included in the non-fungible token (of [CashToken] category) held in this output.
    pub commitment: Commitment,
}

/// see [`Capability`], this struct is used for converting [`TokenData`] to [`CashToken`]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NonFungibleTokenCapability(pub Capability);

///The capability assigned to a particular non-fungible token.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Capability {
    /// No capability – the encoded non-fungible token is an immutable token.
    None = 0x00,
    /// The mutable capability – the encoded non-fungible token is a mutable token.
    Mutable = 0x01,
    /// The minting capability – the encoded non-fungible token is a minting token.
    Minting = 0x02,
}

/// Bitfield trait to get a [`TokenBitfield`] encodings.
pub trait Bitfield {
    /// encodes bitfield number
    fn bitfield(&self) -> u8;
    /// see [`Capability`]
    fn capability(&self) -> u8;
    /// see [`TokenBitFlags::HasAmount`]
    fn has_amount(&self) -> bool;
    /// see [`TokenBitFlags::HasCommitmentLength`]
    fn has_commitment_length(&self) -> bool;
    /// see [`TokenBitFlags::HasNFT`]
    fn has_nft(&self) -> bool;
    /// see [`TokenBitFlags::ReservedBit`]
    fn reserved_bit_is_set(&self) -> bool;
}

/// [`TokenBitfield`]  Encoded two 4-bit fields:
///
///   Prefix_structure (token_bitfield & 0xf0) - 4 bitflags, defined at the higher
///   half of the bitfield, indicating the structure of the token prefix:
///     0x80 (0b10000000) - RESERVED_BIT, must be unset.
///     0x40 (0b01000000) - HAS_COMMITMENT_LENGTH, the prefix encodes a commitment length and commitment.
///     0x20 (0b00100000) - HAS_NFT, the prefix encodes a non-fungible token.
///     0x10 (0b00010000) - HAS_AMOUNT, the prefix encodes an amount of fungible tokens.
///
/// [`Capability`] (token_bitfield & 0x0f) – A 4-bit value, defined at the lower half of
///   the bitfield, indicating the non-fungible token capability, if present.
///
/// If not HAS_NFT: must be 0x00.
/// If HAS_NFT:
///     0x00 – No capability – the encoded non-fungible token is an immutable token.
///     0x01 – The mutable capability – the encoded non-fungible token is a mutable token.
///     0x02 – The minting capability – the encoded non-fungible token is a minting token.
///     Values greater than 0x02 are reserved and must not be used.
///
/// If HAS_COMMITMENT_LENGTH:
///   commitment_length – A commitment length is required (minimally-encoded in CompactUint format2) with a minimum value of 1 (0x01).
///   commitment – The non-fungible token's commitment byte string of commitment_length is required.
///
/// If HAS_AMOUNT:
///   ft_amount – An amount of fungible tokens is required (minimally-encoded in CompactUint format2)
///   with a minimum value of 1 (0x01) and a maximum value equal to the maximum VM number, 9223372036854775807 (0xffffffffffffff7f).
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TokenBitfield(pub u8);

impl Bitfield for TokenBitfield {
    /// Returns the [`TokenBitfield`] of this [`CashToken`]
    fn bitfield(&self) -> u8 {
        let capability = if self.has_nft() { self.0 & 0x0f } else { 0 };
        let nft_bit = if self.has_nft() {
            TokenBitFlags::HasNFT as u8
        } else {
            TokenBitFlags::UnSet as u8
        };
        let has_commitment_len = if self.has_commitment_length() {
            TokenBitFlags::HasCommitmentLength as u8
        } else {
            TokenBitFlags::UnSet as u8
        };
        let amount = if self.has_amount() {
            TokenBitFlags::HasAmount as u8
        } else {
            TokenBitFlags::UnSet as u8
        };
        let token_bitfiled = nft_bit | has_commitment_len | amount | capability as u8;
        token_bitfiled
    }
    fn capability(&self) -> u8 {
        self.0 & TokenBitFlags::NftCapabilityMask as u8
    }
    fn has_amount(&self) -> bool {
        self.0 & TokenBitFlags::HasAmount as u8 != 0
    }
    fn has_commitment_length(&self) -> bool {
        self.0 & TokenBitFlags::HasCommitmentLength as u8 != 0
    }
    //Should be false.
    fn reserved_bit_is_set(&self) -> bool {
        self.0 & TokenBitFlags::ReservedBit as u8 != 0
    }
    fn has_nft(&self) -> bool {
        self.0 & TokenBitFlags::HasNFT as u8 != 0
    }
}

impl Bitfield for CashToken {
    fn bitfield(&self) -> u8 {
        let capability = if self.has_nft() { self.capability() } else { 0 };
        let nft_bit = if self.has_nft() {
            TokenBitFlags::HasNFT as u8
        } else {
            TokenBitFlags::UnSet as u8
        };
        let has_commitment_len = if self.has_commitment_length() {
            TokenBitFlags::HasCommitmentLength as u8
        } else {
            TokenBitFlags::UnSet as u8
        };
        let amount = if self.has_amount() {
            TokenBitFlags::HasAmount as u8
        } else {
            TokenBitFlags::UnSet as u8
        };
        let token_bitfiled = nft_bit | has_commitment_len | amount | capability as u8;
        token_bitfiled
    }
    fn capability(&self) -> u8 {
        self.nft.as_ref().unwrap().capability.0 as u8
    }
    fn has_amount(&self) -> bool {
        self.amount.0 > 0
    }
    fn has_commitment_length(&self) -> bool {
        if self.nft.is_some() {
            self.commitment().0.len() > 0
        } else {
            false
        }
    }
    fn has_nft(&self) -> bool {
        self.nft.is_some()
    }
    fn reserved_bit_is_set(&self) -> bool {
        self.bitfield() & TokenBitFlags::ReservedBit as u8 != 0
    }
}

impl BitcoinSer for TokenBitfield {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        self.0.ser_to(bytes);
    }
    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        Ok(TokenBitfield(BitcoinSer::deser(data)?))
    }
}
impl BitAnd for TokenBitfield {
    type Output = Self;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// The commitment contents included in the [`NFT`]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Commitment(pub Bytes);

impl BitcoinSer for Commitment {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        if self.0.is_empty() {
            ()
        } else {
            self.0.ser_to(bytes)
        }
        
    }
    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        if data.is_empty() {
            Ok(Commitment(Bytes::from(vec![])))
        } else {
            let size = read_compact_uint_minimal(data)? as usize;
            Ok(Commitment(read_bytes(data,size)?))
        }
    }
}

impl BitcoinSer for NonFungibleTokenCapability {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        let capability = self.0 as u8;
        capability.ser_to(bytes);
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        let bytes = BitcoinSer::deser(data)?;
        match bytes {
            0x00 => Ok(NonFungibleTokenCapability(Capability::None)),
            0x01 => Ok(NonFungibleTokenCapability(Capability::Mutable)),
            0x02 => Ok(NonFungibleTokenCapability(Capability::Minting)),
            _ => Err(DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: bytes,
            }),
        }
    }
}

impl BitcoinSer for Option<NFT> {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        match self {
            Some(token) => {
                token.capability.ser_to(bytes);
                token.commitment.ser_to(bytes);
            }
            None => (),
        }
    }

    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        Ok(Some(NFT {
            capability: BitcoinSer::deser(data)?,
            commitment: BitcoinSer::deser(data)?,
        }))
    }
}

impl BitcoinSer for NFT {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        self.capability.ser_to(bytes);
        self.commitment.ser_to(bytes);
    }
    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        Ok(NFT {
            capability: BitcoinSer::deser(data)?,
            commitment: BitcoinSer::deser(data)?,
        })
    }
}

/// Used to deserialize an output containing a [`TOKEN_PREFIX`]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WrappedTokenScript(pub Option<CashToken>, pub Script);

impl BitcoinSer for WrappedTokenScript {
    fn ser_to<S: BitcoinSerializer>(&self, bytes: &mut S) {
        self.0.ser_to(bytes);
    }
    fn deser(data: &mut bytes::Bytes) -> Result<Self, DataError> {
        let wspk = Script::unwrap_prefixed_lockscript(data)?;
        Ok(wspk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{Hashed, Sha256d};
    use bytes::Bytes;

    fn verify_ser(token: CashToken, ser: &[u8]) {
        assert_eq!(token.ser().as_ref(), ser);
        assert_eq!(token.ser_len(), ser.len());
        let mut bytes = Bytes::copy_from_slice(ser);
        assert_eq!(token, CashToken::deser(&mut bytes).unwrap());
    }
    #[test]
    fn test_token_ser() {
        let category_id = Sha256d::from_be_hex(
            "5dc629363b5333b514365d0c247c3ec5f2bc85959bc3480d823d2f3e4b202401",
        ).unwrap();

        let some_token_prefix =
            hex::decode("ef0124204b3e2f3d820d48c39b9585bcf2c53e7c240c5d3614b533533b3629c65d6001ff")
                .unwrap();

        let cashtoken = CashToken {
            amount: CompactUint(0),
            category: category_id.into(),
            nft: Some(NFT {
                capability: NonFungibleTokenCapability(Capability::None),
                commitment: Commitment(Bytes::from([255].as_ref())),
            }),
        };

        verify_ser(cashtoken, &some_token_prefix);
        let category_id = Sha256d::from_be_hex(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ).unwrap();
        let some_token_prefix =
        hex::decode("efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb20")
            .unwrap();
        let token2 = CashToken {
            amount: CompactUint(0),
            category: category_id.into(),
            nft:Some( NFT{
                capability:NonFungibleTokenCapability(Capability::None),
                commitment:Commitment(Bytes::copy_from_slice(vec![].as_ref()) ),
            }),
            
        };
        verify_ser(token2,&some_token_prefix);
    }

    #[test]

    fn test_invalid_prefix() {
   
        // Invalid token encoding
        let some_token_prefix =
            hex::decode("efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb00")
                .unwrap();

        let token_pre = CashToken::deser(&mut some_token_prefix.into());
        assert_eq!(token_pre, Err(DataError::NoTokens { error: "Invalid token prefix: must encode at least one token".to_string() }));
            
        // Invalid token length
        let short_token_prefix = hex::decode("efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
        let token_pre = CashToken::deser(&mut short_token_prefix.into());
        assert_eq!(token_pre, Err(DataError::InvalidTokenPrefixLength { minimum_length: MINIMUM_PREFIX_LENGTH as usize,actual:17 }));
     
    }
}
