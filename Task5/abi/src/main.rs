use ethers::prelude::*;
use eyre::Result;
use std::sync::Arc;

// 1. 使用 abigen! 宏定义合约接口
// 这里我们不需要完整的 JSON ABI，只需要定义我们要调用的方法即可
abigen!(
    ISimpleToken, // 生成的 Rust 结构体名称
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function totalSupply() external view returns (uint256)
    ]"#
);

#[tokio::main]
async fn main() -> Result<()> {
    // 2. 设置 RPC 节点 (这里使用 Arbitrum Sepolia 的公共节点)
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    // 3. 定义目标合约地址 (请替换为你自己在浏览器上找到的地址!)
    // 示例地址：Arbitrum Sepolia 上的某个 ERC20 (这里仅作占位符，务必替换)
    let contract_address = "0x980B62Da83eFf3D4576C647993b0c1D7faf17c73".parse::<Address>()?;

    // 4. 实例化合约
    let contract = ISimpleToken::new(contract_address, client);

    println!("正在连接合约: {:?}", contract_address);

    // 5. 调用只读方法
    // 注意：所有的合约调用都是异步的
    let name = contract.name().call().await?;
    let symbol = contract.symbol().call().await?;
    let total_supply = contract.total_supply().call().await?;

    // 6. 打印结果
    println!("合约名称: {}", name);
    println!("代币符号: {}", symbol);
    println!("总供应量: {}", total_supply);

    Ok(())
}