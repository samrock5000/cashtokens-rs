#[cfg(test)]
mod tests {
    use ::bytes::Bytes;
    use bitcoinsuite_core::tx::{CashToken, MINIMUM_PREFIX_LENGTH};
    use bitcoinsuite_core::{error::DataError, ser::*};
    use serde_json::*;
    use std::fs::File;
    use std::io::Read;

    use bitcoinsuite_core::{
        hash::{Hashed, Sha256d},
        tx::{Capability, Commitment, NonFungibleTokenCapability, TxId, NFT},
    };
    use std::path::Path;

    #[test]
    fn test_prefixes_deser() {
        let mut file = File::open("token-vectors.json").expect("Could not open file");
        let mut buffer = String::new();

        file.read_to_string(&mut buffer).unwrap();

        let parsed_token_data: Value = from_str(&buffer).unwrap();
        let token_data = parsed_token_data.as_array().unwrap();
        let mut tokens_prefix_vec: Vec<Vec<u8>> = vec![];
        let mut tokens_data_vec: Vec<Vec<u8>> = vec![];
        let mut cashtoken_vec: Vec<CashToken> = vec![];

        for item in token_data.iter() {
            for (key, value) in item.as_object().unwrap() {
                match key.as_str() {
                    "prefix" => {
                        assert!(&value.is_string());
                        //Decodes a hex string into raw bytes
                        let token_bytes = hex::decode(value.as_str().unwrap());
                        let token_prefix = Vec::from(token_bytes.unwrap());
                        tokens_prefix_vec.push(token_prefix);
                    }
                    _ => (),
                };
                match key.as_str() {
                    "data" => {
                        let amount: u64 = value["amount"].as_str().unwrap().parse().unwrap();

                        let category = value["category"].clone().as_str().unwrap().to_string();
                        let category = Sha256d::from_be_hex(&category);
                        let nft = value["nft"].clone();
                        let commitment = nft["commitment"].clone();
                        let commitment = match commitment {
                            Value::String(data) => data,
                            _ => "".to_string(),
                        };

                        let commitment = hex::decode(commitment);
                        let commitment =
                            Commitment(Bytes::copy_from_slice(commitment.unwrap().as_ref()));

                        let capability = nft["capability"].clone();

                        let capability = match capability.as_str() {
                            Some("none") => NonFungibleTokenCapability(Capability::None),
                            Some("mutable") => NonFungibleTokenCapability(Capability::Mutable),
                            Some("minting") => NonFungibleTokenCapability(Capability::Minting),

                            //This Works for now.
                            _ => NonFungibleTokenCapability(Capability::None),
                        };

                        if nft.is_null() == true {
                            let cashtoken = CashToken {
                                amount: CompactUint(amount),
                                category: TxId::from(category.unwrap()),
                                nft: None,
                            };
                            cashtoken_vec.push(cashtoken.clone());
                            tokens_data_vec.push(cashtoken.ser().to_vec());
                            
                        } else {
                            let nft = NFT {
                                commitment,
                                capability,
                            };
                            let cashtoken = CashToken {
                                amount: CompactUint(amount),
                                category: TxId::from(category.unwrap()),
                                nft: Some(nft),
                            };
                            cashtoken_vec.push(cashtoken.clone());
                            tokens_data_vec.push(cashtoken.ser().to_vec());
                        }
                    }

                    _ => (),
                };
            }
        }
        assert_eq!(tokens_prefix_vec.iter().eq(tokens_data_vec.iter()), true);

        assert_eq!(tokens_data_vec[60], tokens_prefix_vec[60]);

        let mut prefix_iter = tokens_prefix_vec.iter().enumerate();
        

        //deserializes every "prefix" field from the json file to CashToken and matches it to the cashtoken_vec[index] we collected.
        tokens_prefix_vec.iter().for_each(|prefix| {
            let mut bytes = Bytes::copy_from_slice(prefix);
            assert_eq!(cashtoken_vec[prefix_iter.next().unwrap().0], CashToken::deser(&mut bytes).unwrap());

        } )
    }

    #[test]
    fn test_invalid_prefix() {
        let mut file = File::open("token-prefix-invalid.json").expect("Could not open file");
        let mut buffer = String::new();

        file.read_to_string(&mut buffer).unwrap();

        let parsed_token_data: Value = from_str(&buffer).unwrap();
        let token_data = parsed_token_data.as_array().unwrap();
        let mut tokens_prefix_vec: Vec<Vec<u8>> = vec![];

        for item in token_data.iter() {
            for (key, value) in item.as_object().unwrap() {
                /* let _prefixes =  */
                match key.as_str() {
                    "prefix" => {
                        assert!(&value.is_string());

                        let token_bytes = hex::decode(value.as_str().unwrap());
                        let token_prefix = Vec::from(token_bytes.unwrap());

                        tokens_prefix_vec.push(token_prefix);
                    }
                    _ => (),
                };
            }
        }
        /* {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb00",
          "error": "Invalid token prefix: must encode at least one token. Bitfield: 0b0"
        }, */
        let token = &tokens_prefix_vec[0];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::NoTokens {
                    error: "Invalid token prefix: must encode at least one token".to_string()
                }
        );

        /*   {
          "prefix": "ef",
          "error": "Invalid token prefix: insufficient length. The minimum possible length is 34. Missing bytes: 33"
        }, */

        let token = &tokens_prefix_vec[1];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err()
                == DataError::InvalidTokenPrefixLength {
                    minimum_length: MINIMUM_PREFIX_LENGTH as usize,
                    actual: 1,
                }
        );
        /*  {
          "prefix": "efbbbbbbbb1001",
          "error": "Invalid token prefix: insufficient length. The minimum possible length is 34. Missing bytes: 27"
        }, */
        let token = &tokens_prefix_vec[2];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err()
                == DataError::InvalidTokenPrefixLength {
                    minimum_length: MINIMUM_PREFIX_LENGTH as usize,
                    actual: 7,
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
          "error": "Invalid token prefix: insufficient length. The minimum possible length is 34. Missing bytes: 1"
        }, */
        let token = &tokens_prefix_vec[3];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err()
                == DataError::InvalidTokenPrefixLength {
                    minimum_length: MINIMUM_PREFIX_LENGTH as usize,
                    actual: 33,
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb80",
          "error": "Invalid token prefix: reserved bit is set. Bitfield: 0b10000000"
        }, */
        let token = &tokens_prefix_vec[4];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidPrefixEncoding {
                    error: String::from("Invalid token prefix: reserved bit is set"),
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbba0",
          "error": "Invalid token prefix: reserved bit is set. Bitfield: 0b10100000"
        }, */
        let token = &tokens_prefix_vec[5];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidPrefixEncoding {
                    error: String::from("Invalid token prefix: reserved bit is set"),
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb9001",
          "error": "Invalid token prefix: reserved bit is set. Bitfield: 0b10010000"
        }, */
        let token = &tokens_prefix_vec[6];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err()
                == DataError::InvalidPrefixEncoding {
                    error: String::from("Invalid token prefix: reserved bit is set"),
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb23",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 3"
        }, */
        let token = &tokens_prefix_vec[7];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 3,
            }
        );

        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb24",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 4"
        }, */
        let token = &tokens_prefix_vec[8];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 4,
            }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb25",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 5"
        }, */
        let token = &tokens_prefix_vec[9];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 5,
            }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb26",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 6"
        }, */
        let token = &tokens_prefix_vec[10];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 6,
            }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb27",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 7"
        }, */
        let token = &tokens_prefix_vec[11];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 7,
            }
        );
        /* {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb28",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 8"
        }, */
        let token = &tokens_prefix_vec[12];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 8,
            }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb29",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 9"
        },
        { */
        let token = &tokens_prefix_vec[13];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 9,
            }
        );
        /*    {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2a",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 10"
        },*/
        let token = &tokens_prefix_vec[14];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 10,
            }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2b",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 11"
        },*/
        let token = &tokens_prefix_vec[15];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 11,
            }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2c",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 12"
        }, */
        let token = &tokens_prefix_vec[16];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 12,
            }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2d",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 13"
        }, */
        let token = &tokens_prefix_vec[17];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 13,
            }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2e",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 14"
        }, */
        let token = &tokens_prefix_vec[18];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 14,
            }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2f",
          "error": "Invalid token prefix: capability must be none (0), mutable (1), or minting (2). Capability value: 15"
        }, */
        let token = &tokens_prefix_vec[19];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err() == DataError::InvalidCapability {
                expected:
                    "Invalid token prefix: capability must be none (0), mutable (1), or minting (2)"
                        .to_string(),
                actual: 15,
            }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb1101",
          "error": "Invalid token prefix: capability requires an NFT. Bitfield: 0b10001"
        }, */
        let token = &tokens_prefix_vec[20];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(res.unwrap_err() == DataError::CapabilityWithoutNft);
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb1201",
          "error": "Invalid token prefix: capability requires an NFT. Bitfield: 0b10010"
        }, */
        let token = &tokens_prefix_vec[21];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(res.unwrap_err() == DataError::CapabilityWithoutNft);
        /* {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb40",
          "error": "Invalid token prefix: commitment requires an NFT. Bitfield: 0b1000000"
        }, */
        let token = &tokens_prefix_vec[22];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::CommitmentWithoutNft {
                    error: "Invalid token prefix: commitment requires an NFT".to_string(),
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb5001",
          "error": "Invalid token prefix: commitment requires an NFT. Bitfield: 0b1010000"
        }, */

        let token = &tokens_prefix_vec[23];

        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::CommitmentWithoutNft {
                    error: "Invalid token prefix: commitment requires an NFT".to_string(),
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb60",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: invalid CompactUint. Error reading CompactUint: requires at least one byte."
        }, */
        let token = &tokens_prefix_vec[24];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        // May need to update to CompactUint Error
        assert!(
            res.unwrap_err()
                == DataError::CommitmentLengthZero {
                    error: "Error reading CompactUint: requires at least one byte".to_owned()
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb61",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: invalid CompactUint. Error reading CompactUint: requires at least one byte."
        }, */
        let token = &tokens_prefix_vec[25];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::CommitmentLengthZero {
                    error: "Error reading CompactUint: requires at least one byte".to_owned()
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb62",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: invalid CompactUint. Error reading CompactUint: requires at least one byte."
        }, */
        let token = &tokens_prefix_vec[26];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err()
                == DataError::CommitmentLengthZero {
                    error: "Error reading CompactUint: requires at least one byte".to_owned()
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb60fd0100cc",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: invalid CompactUint. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 3, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[27];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 3
                }
        );

        /* {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb60fe01000000cc",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: invalid CompactUint. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 5, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[28];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        
        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 5
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb60ff0100000000000000cc",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: invalid CompactUint. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 9, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[29];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));

        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 9
                }
        );
        /*    {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb6000",
          "error": "Invalid token prefix: if encoded, commitment length must be greater than 0."
        }, */
        let token = &tokens_prefix_vec[30];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::CommitmentLengthZero {
                    error:
                        "Invalid token prefix: if encoded, commitment length must be greater than 0"
                            .to_string()
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb700001",
          "error": "Invalid token prefix: if encoded, commitment length must be greater than 0."
        }, */
        let token = &tokens_prefix_vec[31];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::CommitmentLengthZero {
                    error:
                        "Invalid token prefix: if encoded, commitment length must be greater than 0"
                            .to_string()
                }
        );
        /* {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb6001",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: insufficient bytes. Required bytes: 1, remaining bytes: 0"
        }, */
        let token = &tokens_prefix_vec[32];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 1,
                    actual: 0
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb6101",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: insufficient bytes. Required bytes: 1, remaining bytes: 0"
        }, */
        let token = &tokens_prefix_vec[33];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 1,
                    actual: 0
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb6102cc",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: insufficient bytes. Required bytes: 2, remaining bytes: 1"
        }, */
        let token = &tokens_prefix_vec[34];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 2,
                    actual: 1
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb6202cc",
          "error": "Invalid token prefix: invalid non-fungible token commitment. Error reading CompactUint-prefixed bin: insufficient bytes. Required bytes: 2, remaining bytes: 1"
        }, */

        let token = &tokens_prefix_vec[35];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 2,
                    actual: 1
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb10",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: requires at least one byte."
        }, */
        let token = &tokens_prefix_vec[36];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidAmountEncoding {
                    error: "Error reading CompactUint: requires at least one byte".to_string(),
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb10fd00",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: insufficient bytes. CompactUint prefix 253 requires at least 3 bytes. Remaining bytes: 2"
        }, */
        let token = &tokens_prefix_vec[37];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 2,
                    actual: 1
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb10fe000000",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: insufficient bytes. CompactUint prefix 254 requires at least 5 bytes. Remaining bytes: 4"
        }, */
        let token = &tokens_prefix_vec[38];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 4,
                    actual: 3
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb10ff00000000000000",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: insufficient bytes. CompactUint prefix 255 requires at least 9 bytes. Remaining bytes: 8"
        }, */
        let token = &tokens_prefix_vec[39];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 8,
                    actual: 7
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb7001cc",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: requires at least one byte."
        }, */
        let token = &tokens_prefix_vec[40];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 1,
                    actual: 0
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb7001ccfd00",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: insufficient bytes. CompactUint prefix 253 requires at least 3 bytes. Remaining bytes: 2"
        }, */
        let token = &tokens_prefix_vec[41];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 2,
                    actual: 1
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb7001ccfe000000",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: insufficient bytes. CompactUint prefix 254 requires at least 5 bytes. Remaining bytes: 4"
        }, */
        let token = &tokens_prefix_vec[42];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 4,
                    actual: 3
                }
        );
        /*  {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb7001ccff00000000000000",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: insufficient bytes. CompactUint prefix 255 requires at least 9 bytes. Remaining bytes: 8"
        }, */
        let token = &tokens_prefix_vec[43];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidLength {
                    expected: 8,
                    actual: 7
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb30",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: requires at least one byte."
        }, */
        let token = &tokens_prefix_vec[44];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidAmountEncoding { error: "Error reading CompactUint: requires at least one byte".to_string() });
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb1000",
          "error": "Invalid token prefix: if encoded, fungible token amount must be greater than 0."
        }, */
        let token = &tokens_prefix_vec[45];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidAmountEncoding {
                    error: "fungible token amount must be greater than 0".to_string(),
                }
        );
        /* {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb3000",
          "error": "Invalid token prefix: if encoded, fungible token amount must be greater than 0."
        }, */
        let token = &tokens_prefix_vec[46];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidAmountEncoding {
                    error: "fungible token amount must be greater than 0".to_string(),
                }
        );
        /* {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb10fd0100",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 3, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[47];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 3
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb10fe01000000",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 5, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[48];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 5
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb10ff00000000000000",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 9, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[49];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 9
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb30fd0100",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 3, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[50];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 3
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb30fe01000000",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 5, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[51];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 5
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb30ff0100000000000000",
          "error": "Invalid token prefix: invalid fungible token amount encoding. Error reading CompactUint: CompactUint is not minimally encoded. Value: 1, encoded length: 9, canonical length: 1"
        }, */
        let token = &tokens_prefix_vec[52];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidCompactUint {
                    canonical_length: 1,
                    encoded_length: 9
                }
        );
        /*   {
          "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb10ff0000000000000080",
          "error": "Invalid token prefix: exceeds maximum fungible token amount of 9223372036854775807. Encoded amount: 9223372036854775808"
        }, */
        let token = &tokens_prefix_vec[53];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidAmountEncoding { 
                    error: "Invalid token prefix: exceeds maximum fungible token amount of 9223372036854775807".to_string(),
             }
        );
        /*   {
        "prefix": "efbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb30ff0000000000000080",
        "error": "Invalid token prefix: exceeds maximum fungible token amount of 9223372036854775807. Encoded amount: 9223372036854775808"
        } */
        let token = &tokens_prefix_vec[54];
        let res = CashToken::deser(&mut Bytes::copy_from_slice(&token));
        assert!(
            res.unwrap_err()
                == DataError::InvalidAmountEncoding { 
                    error: "Invalid token prefix: exceeds maximum fungible token amount of 9223372036854775807".to_string(),
             }
        );
    }

    #[test]
    fn it_works() {
        let path = Path::new("token-vectors.json");
        let mut f = File::open(path).unwrap();
        let mut buffer = String::new();

        f.read_to_string(&mut buffer).unwrap();
        let stream = Deserializer::from_str(&buffer).into_iter::<Value>();
        for value in stream {
            let x = value.map(|x| x[1]["data"].clone());

            println!("{:#?}", x.unwrap());
        }
    }
}
