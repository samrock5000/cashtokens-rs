// Copyright (c) 2023 The Bitcoin developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

//! Module for errors in this crate.

use thiserror::Error;

/// Errors indicating some data doesn't map to some object.
#[derive(Debug, Error, PartialEq)]
pub enum DataError {
    /// Expect a fixed length which was not met.
    #[error("Invalid length, expected {expected} bytes but got {actual} bytes")]
    InvalidLength {
        /// Expected number of bytes.
        expected: usize,
        /// Actual number of bytes.
        actual: usize,
    },

    /// Expected bytes with multiple allowed lengths, none of which were met.
    #[error("Invalid length, expected one of {expected:?} but got {actual} bytes")]
    InvalidLengthMulti {
        /// List of expected number of bytes.
        expected: Vec<usize>,
        /// Actual number of bytes.
        actual: usize,
    },
    /// Hex contains invalid characters, odd length, etc.
    #[error("Invalid hex: {0}")]
    InvalidHex(hex::FromHexError),

    /// Invalid token capability
    #[error("Invalid token capability: capability must be none 0, mutable 1, or minting 2.")]
    InvalidCapability {
        /// [Capability] error
        expected: String,
        /// invalid [Capability]
        actual: u8,
    },
    /// Commitment requires an NFT error
    #[error("Invalid token prefix: commitment requires an NFT")]
    CommitmentWithoutNft {
        /// [`Commitment`] error
        error: String,
    },

    /// Invalid token prefix: if encoded, commitment length must be greater than 0.
    #[error("Invalid token prefix: if encoded, commitment length must be greater than 0")]
    CommitmentLengthZero {
        /// Commitment length error
        error: String,
    },
    /// Invalid token prefix: invalid fungible token amount encoding.
    #[error("Invalid token prefix: invalid fungible token amount encoding")]
    InvalidAmountEncoding {
        /// Amount Encoding Error
        error: String,
    },
    /// CashTokenDecodingError variants.
    #[error("Invalid token prefix: {error:?}")]
    InvalidPrefixEncoding {
        /// Returns the CashTokenDecodingError from caller.
        error: String,
    },
    ///
    #[error("Invalid token prefix: capability requires an NFT")]
    CapabilityWithoutNft,

    #[error("Invalid token prefix: must encode at least one token")]
    /// No token amount error
    NoTokens {
        /// No token amount error
        error: String,
    },
    /// Expect a minimum token length which was not met.
    #[error("Invalid length, minimum: {minimum_length} actual: {actual} bytes")]
    InvalidTokenPrefixLength {
        ///minimum token prefix lengh
        minimum_length: usize,

        /// Actual number of bytes.
        actual: usize,
    },

    /// Error reading CompactUint: CompactUint is not minimally encoded.
    #[error("CompactUint is not minimally encoded: canonical length: {canonical_length} encoded length: {encoded_length} bytes")]
    InvalidCompactUint {
        /// Actual number of bytes.
        canonical_length: usize,
        ///minimum token prefix lengh
        encoded_length: usize,
    },
}
/// Test
// TODO: implement
#[derive(Debug, Error, PartialEq)]
pub enum CommitmentLengthError {
    /// Return Error with implementor info for [`DataError::InvalidLength`]
    #[error("Invalid length")]
    CommitmentLengthError {
        /// Todo
        error: DataError,
    },
}
