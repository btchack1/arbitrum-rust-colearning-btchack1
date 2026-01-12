use anyhow::{Context, Result};
use ethers::prelude::*;
use ethers::utils::{format_ether, format_units};


const TRANSFER_GAS_CAP: u64 = 21_000;
#[derive(Debug)]
struct FeeSnapshot {
    unit_price_wei: U256,   
    gas_cap: U256,         
    total_fee_wei: U256,   
    total_fee_eth: String,  
    unit_price_gwei: String 
}

// - 动态获取 gas price：provider.get_gas_price()
// - 基础转账 gas_limit：21000
// - total_fee_wei = unit_price_wei * gas_cap
async fn calc_transfer_fee(rpc_endpoint: &str) -> Result<FeeSnapshot> {
    let rpc_client = Provider::<Http>::try_from(rpc_endpoint)
        .with_context(|| format!("RPC URL 无法解析或初始化 Provider: {rpc_endpoint}"))?;

    // 动态获取实时 gas price（非硬编码）
    let unit_price_wei = rpc_client
        .get_gas_price()
        .await
        .context("RPC 调用失败：get_gas_price")?;

    let gas_cap = U256::from(TRANSFER_GAS_CAP);

    // Gas Fee(wei) = Gas Price(wei/gas) × Gas Limit(gas)
    let total_fee_wei = unit_price_wei * gas_cap;

    let unit_price_gwei =
        format_units(unit_price_wei, "gwei").context("format_units 失败：wei -> gwei")?;
    let total_fee_eth = format_ether(total_fee_wei);

    Ok(FeeSnapshot {
        unit_price_wei,
        gas_cap,
        total_fee_wei,
        total_fee_eth,
        unit_price_gwei,
    })
}

fn show_fee_report(rpc_endpoint: &str, snap: &FeeSnapshot) {
    println!("\n================ Arbitrum Transfer Fee Report ================");
    println!("RPC Endpoint   : {rpc_endpoint}");
    println!("--------------------------------------------------------------");
    println!(
        "{:<18} {:>34}",
        "Gas Price (wei)",
        format!("{} wei/gas", snap.unit_price_wei)
    );
    println!(
        "{:<18} {:>34}",
        "Gas Price (gwei)",
        format!("{} gwei/gas", snap.unit_price_gwei)
    );
    println!(
        "{:<18} {:>34}",
        "Gas Limit",
        format!("{} gas", snap.gas_cap)
    );
    println!("--------------------------------------------------------------");
    println!(
        "{:<18} {:>34}",
        "Estimated Fee (wei)",
        format!("{}", snap.total_fee_wei)
    );
    println!(
        "{:<18} {:>34}",
        "Estimated Fee (ETH)",
        format!("~{} ETH", snap.total_fee_eth)
    );
    println!("==============================================================\n");
}

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_endpoint = "https://arbitrum-sepolia-rpc.publicnode.com";

    let snap = calc_transfer_fee(rpc_endpoint).await?;
    show_fee_report(rpc_endpoint, &snap);

    Ok(())
}
