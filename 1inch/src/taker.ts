import { Contract, JsonRpcProvider, parseEther, TransactionRequest } from "ethers"
import { CONFIG, WETH_ABI } from "./config"
import { Wallet } from "./wallet"
import fs from "fs"
import { Resolver } from "./resolver"
import { EscrowFactory } from './escrow-factory'
import { Address, EscrowFactory as OneInchEscrowFactory } from '@1inch/cross-chain-sdk'
import { OrderData } from "./interface"

main().then(() => {
    console.log("Cross-chain order created successfully.")
}).catch((error) => {
    console.error("Error creating cross-chain order:", error)
})

async function main() {
    const provider = new JsonRpcProvider(CONFIG.networkRPC, CONFIG.chains.evm, {
        cacheTimeout: -1,
        staticNetwork: true
    })

    const resolverContract = new Resolver(CONFIG.contracts.resolverContract, CONFIG.contracts.resolverBitcoinContract)
    const resolverWallet = new Wallet(CONFIG.resolverPk, provider)

    // Initialize resolver contracts
    await initialize_resolve_contracts(resolverWallet)

    console.log("Resolver contract balance before order:", await get_resolver_contract_balance(resolverWallet))

    // Read the order tx data from `order.json`
    const orderData: OrderData = JSON.parse(await fs.promises.readFile("order.json", "utf-8"))
    const orderHash = orderData.orderHash
    const resolverTxnData = orderData.resolverTxnData

    // Resolve the user order
    const transactionRequest: TransactionRequest = {
        to: resolverContract.srcAddress,
        data: resolverTxnData
    }

    const { txHash: orderFillHash, blockHash: srcDeployBlock } = await resolverWallet.send_new(
        transactionRequest
    )
    console.log(`Order Fill Tx Hash: ${orderFillHash} and block has ${srcDeployBlock}`);
    console.log(`[EVM]`, `Order ${orderHash} filled in tx ${orderFillHash}`);

    const escrowFactory = new EscrowFactory(provider, CONFIG.contracts.escrowFactory)
    const srcEscrowEvent = await escrowFactory.getSrcDeployEvent(srcDeployBlock)

    const ESCROW_SRC_IMPLEMENTATION = await escrowFactory.getSourceImpl()
    const srcEscrowAddress = new OneInchEscrowFactory(new Address(CONFIG.contracts.escrowFactory)).getSrcEscrowAddress(
        srcEscrowEvent[0],
        ESCROW_SRC_IMPLEMENTATION
    )
    console.log(`Source Escrow Address: ${srcEscrowAddress.toString()}`)

    // Sleep for 10 seconds to simulate the time locks
    await new Promise(resolve => setTimeout(resolve, 10000));


    console.log(`[EVM]`, `Withdrawing funds for resolver from ${srcEscrowAddress}`)
    const txn =
        resolverContract.withdraw('src', srcEscrowAddress, orderData.secret, srcEscrowEvent[0]);

    const { txHash: resolverWithdrawHash } = await resolverWallet.send(
        txn
    )
    console.log(
        `[EVM]`,
        `Withdrew funds for resolver from ${srcEscrowAddress} to ${resolverContract.srcAddress} in tx ${resolverWithdrawHash}`
    )
    console.log("Resolver contract balance after order:", await get_resolver_contract_balance(resolverWallet))

}

async function initialize_resolve_contracts(resolver_wallet: Wallet) {
    const contract_bal = await get_resolver_contract_balance(resolver_wallet)
    if (contract_bal < parseEther("0.1")) {
        const wethContract = new Contract(CONFIG.tokens.WETH, WETH_ABI, resolver_wallet.signer)
        await wethContract.deposit({ value: parseEther("0.1") }).then(tx => tx.wait()); // Deposit 0.1 ETH to WETH contract
        await wethContract.transfer(CONFIG.contracts.resolverContract, parseEther("0.1")).then(tx => tx.wait());
    }
}


async function get_resolver_contract_balance(resolver_wallet: Wallet) {
    const wethContract = new Contract(CONFIG.tokens.WETH, WETH_ABI, resolver_wallet.signer)
    const balance = await wethContract.balanceOf(CONFIG.contracts.resolverContract)
    return balance
}