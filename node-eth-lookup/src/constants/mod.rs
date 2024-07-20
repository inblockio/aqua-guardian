pub const INFURA_KEY_MUST_BE_SET: &str = "INFURA_API_KEY must be set";
pub const ALCHEMY_API_KEY_MUST_BE_SET: &str = "ALCHEMY_API_KEY must be set";
pub const SELF_HOSTED_API_KEY_MUST_BE_SET: &str = "SELF_HOSTED_API_KEY must be set";
pub const FAILED_TO_CREATE_PROVIDER: &str = "Failed to create provider";
pub const UNSUPPORTED_CHAIN_ID: &str = "Unsupported chain ID";
pub const FAILED_TO_GET_CHAIN_ID: &str = "Failed to get chain ID";
pub const SIGNER_PRIVATE_KEY_MUST_BE_SET: &str = "SIGNER_PRIVATE_KEY must be set";
pub const FAILED_TO_PARSE_WALLET_KEY: &str = "Failed to parse wallet key";
pub const FAILED_TO_PARSE_TRANSACTION_HASH: &str = "Failed to parse transaction hash";
pub const FAILED_TO_GET_TRANSACTION: &str = "Failed to get transaction";
pub const TRANSACTION_NOT_FOUND: &str = "Transaction not found";
pub const FAILED_TO_DESERIALIZE_TRANSACTION: &str = "Failed to deserialize transaction";
pub const FAILED_TO_PARSE_BLOCK_NUMBER: &str = "Failed to parse block number";
pub const FAILED_TO_GET_BLOCK: &str = "Failed to get block";
pub const BLOCK_NOT_FOUND: &str = "Block not found";
pub const FAILED_TO_DESERIALIZE_BLOCK_TIME: &str = "Failed to deserialize block time";
pub const FAILED_TO_PARSE_TIMESTAMP: &str = "Failed to parse timestamp";
pub const FAILED_TO_PARSE_INPUT: &str = "Failed to parse input";

pub const WRONG_BLOCK_TIMESTAMP: &str = "Wrong block timestamp";
pub const WRONG_INPUT: &str = "Wrong Input";

#[derive(Debug, Clone, Copy)]
pub enum UrlProvider {
    Infura,
    SelfHosted,
    Alchemy,
}
