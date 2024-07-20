# aqua-eth-lookup
A rust crate implementing the functionality to lookup transactions which contain witness data with the option to use Infura, Alchemy as API providers or to ise a self-hosted Ethereum node.

Infura (implemented as provider)
Ensure to set the .env virable in your infura_rs folder and configure   

SIGNER_PRIVATE_KEY=your_private_wallet_key  
INFURA_API_KEY=your_infura_api_key  
