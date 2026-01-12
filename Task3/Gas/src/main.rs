// 导入核心依赖，保持极简
use ethers::prelude::*;
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ===================== 配置常量 =====================
    /// Arbitrum Sepolia测试网RPC节点地址
    const ARBITRUM_SEPOLIA_RPC: &str = "https://sepolia-rollup.arbitrum.io/rpc";
    /// 以太坊/Arbitrum普通转账通用Gas限额（行业标准值）
    const STANDARD_TRANSFER_GAS_LIMIT: u64 = 21_000;
    /// Wei转Gwei的换算系数（1 Gwei = 10^9 Wei）
    const WEI_TO_GWEI_COEFFICIENT: u64 = 1_000_000_000;

    // ===================== 核心逻辑 =====================
    // 1. 初始化Arbitrum Sepolia测试网Provider
    let arb_sepolia_provider = Provider::<Http>::try_from(ARBITRUM_SEPOLIA_RPC)?;
    println!("✅ 成功连接Arbitrum Sepolia测试网节点");

    // 2. 动态获取实时Gas价格（单位：Wei）
    let current_gas_price_wei = arb_sepolia_provider.get_gas_price().await?;
    println!("\n📊 实时Gas价格信息:");
    println!("   - Gas价格 (Wei): {}", current_gas_price_wei);

    // 3. 打印标准转账Gas限额（固定值）
    println!("   - 标准转账Gas限额: {} gas", STANDARD_TRANSFER_GAS_LIMIT);

    // 4. 计算预估转账手续费（核心公式：手续费 = Gas价格 × Gas限额）
    let estimated_fee_wei = current_gas_price_wei * U256::from(STANDARD_TRANSFER_GAS_LIMIT);
    let estimated_fee_gwei = estimated_fee_wei / U256::from(WEI_TO_GWEI_COEFFICIENT);

    // ===================== 结果输出 =====================
    println!("\n💸 预估转账手续费:");
    println!("   - 手续费 (Wei): {}", estimated_fee_wei);
    println!("   - 手续费 (Gwei): {}", estimated_fee_gwei);
    // 额外补充Ether单位（更易理解）
    let estimated_fee_ether = estimated_fee_wei.as_u128() as f64 / 1e18;
    println!("   - 手续费 (Ether): {:.10} ETH", estimated_fee_ether);

    Ok(())
}