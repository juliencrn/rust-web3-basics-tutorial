use std::env;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use web3::contract::{Contract, Options};
use web3::types::{Address, H160, U256};

#[tokio::main]
async fn main() -> web3::Result<()> {
    dotenv::dotenv().ok();

    // Initiate web3
    let websocket = web3::transports::WebSocket::new(&env::var("RPC_URL").unwrap()).await?;
    let web3s = web3::Web3::new(websocket);

    // Get accounts
    let mut accounts = web3s.eth().accounts().await?;
    accounts.push(H160::from_str(&env::var("ACCOUNT_ADDRESS").unwrap()).unwrap());
    println!("Accounts: {:?}", accounts);

    // for account in accounts {
    //     let balance = web3s.eth().balance(account, None).await?;
    //     println!("ETH balance of {:?}: {}", account, wei_to_eth(balance),)
    // }

    // Get AAVE contract
    let aave_addr = Address::from_str("0x42447d5f59d5bf78a82c34663474922bdf278162").unwrap();
    let aave_token_contract = Contract::from_json(
        web3s.eth(),
        aave_addr,
        include_bytes!("aave_erc20_abi.json"),
    )
    .unwrap();

    let token_name: String = aave_token_contract
        .query("name", (), None, Options::default(), None)
        .await
        .unwrap();

    let total_supply: U256 = aave_token_contract
        .query("totalSupply", (), None, Options::default(), None)
        .await
        .unwrap();

    println!("Token name: {}, total supply: {}", token_name, total_supply);

    // Instantiate Uniswap v2 router
    let router_addr = Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D").unwrap();
    let router_contract = Contract::from_json(
        web3s.eth(),
        router_addr,
        include_bytes!("uniswap_v2_router02_abi.json"),
    )
    .unwrap();

    // Get WETH token
    let weth_addr: Address = router_contract
        .query("WETH", (), None, Options::default(), None)
        .await
        .unwrap();

    println!("WETH address: {:?}", &weth_addr);

    // get estimated gas fees
    let gas_price = web3s.eth().gas_price().await.unwrap();
    println!("gas price: {}", gas_price);

    let dai_addr = Address::from_str("0xc7ad46e0b8a400bb3c915120d284aafba8fc4735").unwrap();
    let valid_timestamp = get_valid_timestamp(5 * 60 * 1000);
    println!("time in ms: {}", valid_timestamp);

    let out_gas_estimate = router_contract
        .estimate_gas(
            "swapExactETHForTokens",
            (
                // how much DAI we want to get out of the swap
                U256::from_dec_str("106662000000").unwrap(),
                vec![weth_addr, dai_addr],
                accounts[0],
                U256::from_dec_str(&valid_timestamp.to_string()).unwrap(),
            ),
            accounts[0],
            Options {
                // amount of ETH we will put into the swap (10**18/20 => 0.05 eth).
                value: Some(U256::exp10(18).checked_div(20.into()).unwrap()),
                gas: Some(500_000.into()),
                ..Default::default()
            },
        )
        .await
        .expect("Error");

    println!("estimated gas amount: {}", out_gas_estimate);

    Ok(())
}

fn wei_to_eth(wei_value: U256) -> f64 {
    (wei_value.as_u128() as f64) / 1_000_000_000_000_000_000.0
}

fn get_valid_timestamp(future_millis: u128) -> u128 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let time_millis = since_epoch.as_millis().checked_add(future_millis).unwrap();
    time_millis
}
