use anyhow::{Context, Result};
use ethers::prelude::*;
use ethers::utils::{format_ether, to_checksum};
use std::{env, str::FromStr};

const DEFAULT_RPC: &str = "https://sepolia-rollup.arbitrum.io/rpc";
const DEFAULT_ADDRESS: &str = "0xeB9aB700eE3EdC24Faf6094D893A42A5464313dE";

#[derive(Debug)]
struct Config {
    address: Address,
    rpc_url: String,
}

impl Config {
    fn from_args_or_default() -> Result<Self> {

        let mut args = env::args().skip(1);

        let address_str = args.next().unwrap_or_else(|| DEFAULT_ADDRESS.to_string());
        let rpc_url = args.next().unwrap_or_else(|| DEFAULT_RPC.to_string());

        let address = Address::from_str(&address_str)
            .with_context(|| format!("地址格式不正确: {address_str}"))?;

        Ok(Self { address, rpc_url })
    }
}

async fn query_balance(provider: &Provider<Http>, address: Address) -> Result<U256> {
    provider
        .get_balance(address, None)
        .await
        .context("RPC 调用失败：get_balance")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_args_or_default()?;

    let provider = Provider::<Http>::try_from(cfg.rpc_url.as_str())
        .with_context(|| format!("RPC URL 无法解析或初始化 Provider: {}", cfg.rpc_url))?;

    let wei = query_balance(&provider, cfg.address).await?;
    let eth = format_ether(wei);
    let addr_checksum = to_checksum(&cfg.address, None);

    println!("================ BALANCE RESULT ================");
    println!("RPC     : {}", cfg.rpc_url);
    println!("Address : {}", addr_checksum);
    println!("-----------------------------------------------");
    println!("Balance: {eth} ETH ");
    println!("Balance: {wei} wei");
    println!("{} wei = {} ETH", wei, eth);
    println!("===============================================");

    Ok(())
}
