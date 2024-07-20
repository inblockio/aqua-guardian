//tests for the infura provider
use ethereum_types::H512;
use node_eth_lookup::{
    constants::{
        UrlProvider, FAILED_TO_PARSE_TRANSACTION_HASH, WRONG_BLOCK_TIMESTAMP, WRONG_INPUT,
    },
    providers::get_tx_data,
};

#[tokio::test]
async fn test_get_tx_data_success_sepolia() {
    // Chain ID and transaction hash for a successful scenario (Sepolia)
    let chain_id = 0xaa36a7;
    let tx_hash = "0xd82cb4b91a83124fdd2aa367256c22b94276cbc046d1cf56379035fb13a9dd00";
    let correct_input: H512 = "0xe41d6466e2f1deb48afd31993a6b6e84b50185d2f30b399d97a801b0cf82e35764d52b39920ac39f11e518fc3f482d68d04e3ebaff91081dad13d80ac41c069a".parse().unwrap();

    // Call the function and unwrap the result
    let result = get_tx_data(chain_id, tx_hash, UrlProvider::Infura)
        .await
        .unwrap();

    // Verify the result
    assert_eq!(result.0, correct_input, "{}", WRONG_INPUT);
    assert_eq!(result.1, 1717611456, "{}", WRONG_BLOCK_TIMESTAMP);
}

#[tokio::test]
async fn test_get_tx_data_success_mainnet() {
    // Chain ID and transaction hash for a successful scenario (Sepolia)
    let chain_id = 0x1;
    let tx_hash = "0x9d4897d3e381982ee872cb193469d991cce8d087f0cd5fe275926f80c1326a1e";
    let correct_input: H512 = "0x07dbf300856866592aaa5a26c4fa55db82fab0cc55ee2f3380aeb42c58c3bd22b637134bbc93d744cc6f040761114e68a4bff8b4425884e75e1a8aca946e0432".parse().unwrap();

    // Call the function and unwrap the result
    let result = get_tx_data(chain_id, tx_hash, UrlProvider::Infura)
        .await
        .unwrap();

    // Verify the result
    assert_eq!(result.0, correct_input, "{}", WRONG_INPUT);
    assert_eq!(result.1, 1713010739, "{}", WRONG_BLOCK_TIMESTAMP);
}

#[tokio::test]
async fn test_get_tx_data_success_holesky() {
    // Chain ID and transaction hash for a successful scenario (Sepolia)
    let chain_id = 0x4268;
    let tx_hash = "0xe20ee33fe150423099d6c22bf84683e19d03e40371e2c76e59293d026e8d0101";
    let correct_input: H512 = "0x68f827d377cfccb19fe26fd9e9dd57627cc39f299411ae7192a4c3a5842e94ff38e3ed423cc6d16cbb7627b462a219381387a65bfc8eb8b623f0db6913ae0ef1".parse().unwrap();

    // Call the function and unwrap the result
    let result = get_tx_data(chain_id, tx_hash, UrlProvider::Infura)
        .await
        .unwrap();

    // Verify the result
    assert_eq!(result.0, correct_input, "{}", WRONG_INPUT);
    assert_eq!(result.1, 1716190800, "{}", WRONG_BLOCK_TIMESTAMP);
}

#[tokio::test]
async fn test_get_tx_data_invalid_chain_id() {
    // Invalid chain ID but valid transaction hash
    let chain_id = 0x2;
    let tx_hash = "0xd82cb4b91a83124fdd2aa367256c22b94276cbc046d1cf56379035fb13a9dd00";

    // Call the function and expect an error
    let result = get_tx_data(chain_id, tx_hash, UrlProvider::Infura).await;

    // Verify that the result is an error
    assert!(result.is_err());

    // Optionally, check the specific error message or type

    assert_eq!(
        format!("{}", result.err().unwrap()),
        "Unsupported chain ID: 2",
        "Invalid chain ID test failed"
    );
}

#[tokio::test]
async fn test_get_tx_data_invalid_tx_hash() {
    // Valid chain ID but invalid transaction hash
    let chain_id = 0xaa36a7;
    let tx_hash = "invalid_tx_hash";

    // Call the function and expect an error
    let result = get_tx_data(chain_id, tx_hash, UrlProvider::Infura).await;

    // Verify that the result is an error
    assert!(result.is_err());
    assert_eq!(
        format!("{}", result.err().unwrap()),
        FAILED_TO_PARSE_TRANSACTION_HASH,
        "Invalid transaction hash test failed"
    );
}
