import { JsonRpcProvider, Contract, formatEther } from "ethers"
import { CONFIG, WETH_ABI } from "./config"
import { Wallet } from "./wallet"

main().then(() => {
    console.log("✅ Balance check completed successfully.")
}).catch((error) => {
    console.error("❌ Error checking balances:", error)
})

async function main() {
    const provider = new JsonRpcProvider(CONFIG.networkRPC, CONFIG.chains.evm, {
        cacheTimeout: -1,
        staticNetwork: true
    })

    const userWallet = new Wallet(CONFIG.userPk, provider)
    const userAddress = await userWallet.getAddress()

    const wethContract = new Contract(CONFIG.tokens.WETH, WETH_ABI, provider)
    
    const latestBlock = await provider.getBlock('latest')
    const userBalance = await wethContract.balanceOf(userAddress)
    const resolverBalance = await wethContract.balanceOf(CONFIG.contracts.resolverContract)

    console.log("⛓️  Blockchain Info:")
    console.log("================")
    console.log(`🌐 Network: ${CONFIG.networkRPC}`)
    console.log(`🆔 Chain ID: ${CONFIG.chains.evm}`)
    console.log(`📦 Block: ${latestBlock!.number}`)
    console.log(`⏰ Timestamp: ${latestBlock!.timestamp}`)
    console.log()
    console.log("💰 WETH Balance Report:")
    console.log("===================")
    console.log(`👤 User Address (${userAddress}):`)
    console.log(`  💎 WETH Balance: ${formatEther(userBalance)} WETH`)
    console.log()
    console.log(`🤖 Resolver Contract (${CONFIG.contracts.resolverContract}):`)
    console.log(`  💎 WETH Balance: ${formatEther(resolverBalance)} WETH`)
}