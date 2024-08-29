use eyre::Result;
use node_eth_lookup::{constants::UrlProvider, providers::get_tx_data};

#[tokio::main]
async fn main() -> Result<()> {
    //mainnet
    //Ok(get_tx_data(0x1,"0x9d4897d3e381982ee872cb193469d991cce8d087f0cd5fe275926f80c1326a1e").await?)
    //holesky
    //Ok(get_tx_data(0x4268,"0xe20ee33fe150423099d6c22bf84683e19d03e40371e2c76e59293d026e8d0101").await?)
    //sepolia
    //Ok(get_tx_data(0xaa36a7,"0xae9b476d8eed73897b0f71ac59c267856dbae64f249518fea862377208436cc5").await?)

    // Sepolia
    // let result = get_tx_data(
    //     0xaa36a7,
    //     "0xd82cb4b91a83124fdd2aa367256c22b94276cbc046d1cf56379035fb13a9dd00",
    //     UrlProvider::Infura,
    // )
    // .await;

    // Mainnet
    // let result = get_tx_data(
    //     0x1,
    //     "0x9d4897d3e381982ee872cb193469d991cce8d087f0cd5fe275926f80c1326a1e",
    //     UrlProvider::Infura,
    // )
    // .await;

    // Holesky
    let result = get_tx_data(
        0x4268,
        "0xe20ee33fe150423099d6c22bf84683e19d03e40371e2c76e59293d026e8d0101",
        UrlProvider::Infura,
    )
    .await;

    match result {
        Ok((input, blocktime_u64)) => {
            println!("Input: {:?}", input);
            println!("Block Time: {}", blocktime_u64);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            // Handle the error further if needed
        }
    }

    Ok(())
}
