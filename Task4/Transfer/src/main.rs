use anyhow::{anyhow, Context, Result};
use dotenvy::dotenv;
use ethers::prelude::*;
use ethers::providers::Middleware;
use ethers::types::transaction::eip1559::Eip1559TransactionRequest;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::utils::{format_ether, parse_units};
use std::{env, str::FromStr, sync::Arc, time::Duration};

const ARB_SEPOLIA_CHAIN_ID: u64 = 421_614;

fn must_env(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("缺少环境变量：{key}"))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let rpc_url = must_env("ARB_RPC_URL")?;
    let privkey = must_env("SENDER_PRIVKEY")?;
    let to_addr_raw = must_env("TO_ADDRESS")?;
    let amount_raw = env::var("AMOUNT_ETH").unwrap_or_else(|_| "0.0005".to_string());

    // 1) 地址校验
    let to: Address = to_addr_raw
        .parse()
        .with_context(|| format!("TO_ADDRESS 不是合法 EVM 地址：{to_addr_raw}"))?;

    if to == Address::zero() {
        return Err(anyhow!("TO_ADDRESS 不能是零地址"));
    }

    // 2) 连接 RPC
    let provider = Provider::<Http>::try_from(rpc_url.as_str())
        .with_context(|| format!("RPC URL 无法初始化 Provider：{rpc_url}"))?
        .interval(Duration::from_millis(250));

    // 3) 钱包（环境变量私钥）+ ChainId
    let wallet = LocalWallet::from_str(privkey.trim())
        .context("私钥解析失败（请确认是 0x 开头的 hex）")?
        .with_chain_id(ARB_SEPOLIA_CHAIN_ID);

    let from = wallet.address();

    if from == to {
        return Err(anyhow!("转账地址不能是自己：from == to"));
    }

    // 4) 组装 Client
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    // 5) 金额（ETH -> wei）
    let value_wei: U256 = parse_units(&amount_raw, 18)
        .with_context(|| format!("AMOUNT_ETH 无法解析：{amount_raw}"))?
        .into();

    // 6) 构造交易：优先 EIP-1559；失败 fallback Legacy(gas_price)
    let mut tx: TypedTransaction = match provider.estimate_eip1559_fees(None).await {
        Ok((max_fee, max_tip)) => {
            let mut r = Eip1559TransactionRequest {
                from: Some(from),
                to: Some(NameOrAddress::Address(to)),
                value: Some(value_wei),
                ..Default::default()
            };
            r.max_fee_per_gas = Some(max_fee);
            r.max_priority_fee_per_gas = Some(max_tip);
            r.into()
        }
        Err(_) => {
            let gp = provider
                .get_gas_price()
                .await
                .context("RPC 调用失败：get_gas_price")?;

            // Legacy 交易才有 gas_price
            let r = TransactionRequest::new()
                .from(from)
                .to(to)
                .value(value_wei)
                .gas_price(gp);

            r.into()
        }
    };

    // 7) gas limit：优先 estimate_gas；失败则用基础转账常见值 21,000
    let gas_limit = match client.estimate_gas(&tx, None).await {
        Ok(gl) => gl,
        Err(_) => U256::from(21_000u64),
    };

    // 把 gas_limit 写回 tx
    match &mut tx {
        TypedTransaction::Legacy(r) => r.gas = Some(gas_limit),
        TypedTransaction::Eip1559(r) => r.gas = Some(gas_limit),
        _ => {}
    }

    // 8) 预估费用
    let est_fee_wei = match &tx {
        TypedTransaction::Eip1559(r) => gas_limit * r.max_fee_per_gas.unwrap_or_else(U256::zero),
        TypedTransaction::Legacy(r) => gas_limit * r.gas_price.unwrap_or_else(U256::zero),
        _ => U256::zero(),
    };

    // 9) 打印信息
    println!("=== Arbitrum Sepolia ETH Transfer ===");
    println!("From   : {from:?}");
    println!("To     : {to:?}");
    println!("Amount : {} ETH", amount_raw);
    println!("GasLim : {gas_limit}");

    match &tx {
        TypedTransaction::Eip1559(r) => {
            println!("Type   : EIP-1559");
            if let Some(m) = r.max_fee_per_gas {
                println!("MaxFee : {} wei", m);
            }
            if let Some(tip) = r.max_priority_fee_per_gas {
                println!("Tip    : {} wei", tip);
            }
        }
        TypedTransaction::Legacy(r) => {
            println!("Type   : Legacy");
            if let Some(gp) = r.gas_price {
                println!("GasPrice: {} wei", gp);
            }
        }
        _ => println!("Type   : Other"),
    }

    println!("EstFee : {} ETH (rough)", format_ether(est_fee_wei));

    // 10) 发送交易（签名+广播）
    let pending = client
        .send_transaction(tx, None)
        .await
        .context("发送交易失败：send_transaction")?;

    let tx_hash = *pending;
    println!("\nTxHash : {tx_hash:?}");
    println!("Explorer: https://sepolia.arbiscan.io/tx/{tx_hash}\n");

    // 11) 等待回执
    let receipt = pending
        .await
        .context("等待交易回执失败（可能网络问题或交易被丢弃）")?
        .ok_or_else(|| anyhow!("交易未返回回执（可能 pending 太久或被替换）"))?;

    println!("=== Receipt ===");
    println!("Status : {:?}", receipt.status);
    println!("Block  : {:?}", receipt.block_number);
    println!("GasUsed: {:?}", receipt.gas_used);
    println!("Done ✅");

    Ok(())
}
