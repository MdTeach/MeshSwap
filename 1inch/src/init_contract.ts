import { JsonRpcProvider, Contract, formatEther, parseEther } from "ethers"
import { CONFIG, WETH_ABI } from "./config"
import { Wallet } from "./wallet"
import { ensureWethAndApproval } from "./utils"

main().then(() => {
}).catch((error) => {
    console.error("❌ Error initing the contract balances:", error)
})

async function main() {
    const provider = new JsonRpcProvider(CONFIG.networkRPC, CONFIG.chains.evm, {
        cacheTimeout: -1,
        staticNetwork: true
    })

    // Ensure maker wallet has WETH and approval
    const maker_wallet = new Wallet(CONFIG.userPk, provider)
    await ensureWethAndApproval(
        maker_wallet,
        CONFIG.tokens.WETH,
        CONFIG.contracts.limitOrderContract
    )

    // Ensure resolver wallet has WETH and approval
    const resolver_wallet = new Wallet(CONFIG.resolverPk, provider)
    await init_resolver_contract(resolver_wallet)

    console.log("✅ Contract balances initialized successfully.")
}


async function init_resolver_contract(resolver_wallet: Wallet) {
    const wethContract = new Contract(CONFIG.tokens.WETH, WETH_ABI, resolver_wallet.signer)
    const dustAmount = parseEther("0.01");
    await wethContract.deposit({ value: dustAmount }).then(tx => tx.wait()); // Deposit 0.1 ETH to WETH contract
    await wethContract.transfer(CONFIG.contracts.resolverContract, dustAmount).then(tx => tx.wait());
}

