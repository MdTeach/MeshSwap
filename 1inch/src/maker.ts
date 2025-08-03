import { JsonRpcProvider, MaxUint256, parseEther } from "ethers"
import { TakerTraits, CrossChainOrder, AmountMode, Address, HashLock, TimeLocks, AuctionDetails, randBigInt, EscrowFactory as EF } from '@1inch/cross-chain-sdk'
import { CONFIG } from "./config"
import { Wallet } from "./wallet"
import { ensureWethAndApproval } from "./utils"
import { UINT_40_MAX } from "@1inch/byte-utils"
import { writeFileSync, readFileSync } from "fs"
import { Resolver } from "./resolver"
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

    const userWallet = new Wallet(CONFIG.userPk, provider)
    const userAddress = await userWallet.getAddress()

    await ensureWethAndApproval(
        userWallet,
        CONFIG.tokens.WETH,
        CONFIG.contracts.limitOrderContract
    )

    const amount_eth_to_send = parseEther('1')
    const amount_btc_to_receive = parseEther('0.01')
    const publicParams = JSON.parse(readFileSync('../prover/script/public_params.json', 'utf8'))
    const hashLockSecret = publicParams.secret_hash
    const startTime = BigInt((await provider.getBlock('latest'))!.timestamp)

    const order_params: OrderParams = {
        makerAddress: userAddress,
        amountMaking: amount_eth_to_send,
        amountTaking: amount_btc_to_receive,
        hashLockSecret: hashLockSecret,
        startTime: startTime
    }


    const order = make_order(order_params)
    const orderHash = order.getOrderHash(CONFIG.chains.evm)
    const signature = await userWallet.signOrder(CONFIG.chains.evm, order)

    const resolverContract = new Resolver(CONFIG.contracts.resolverContract, CONFIG.contracts.resolverBitcoinContract)
    const fillAmount = order.makingAmount;
    const resolverTxnData = resolverContract.deploySrc(
        CONFIG.chains.evm,
        order,
        signature,
        TakerTraits.default()
            .setExtension(order.extension)
            .setAmountMode(AmountMode.maker)
            .setAmountThreshold(order.takingAmount),
        fillAmount
    );

    const orderData:OrderData = {
        orderHash: orderHash,
        resolverTxnData: resolverTxnData.data!,
        secret: hashLockSecret,
        makerAmount: order.makingAmount.toString(),
        takerAmount: order.takingAmount.toString()
    }

    writeFileSync('order.json', JSON.stringify(orderData, null, 2));

}

interface OrderParams {
    makerAddress: string,
    amountMaking: bigint,
    amountTaking: bigint,
    hashLockSecret: string,
    startTime: bigint
}

function make_order(params: OrderParams): CrossChainOrder {
    return CrossChainOrder.new(
        new Address(CONFIG.contracts.escrowFactory),
        {
            salt: randBigInt(1000n),
            maker: new Address(params.makerAddress),
            makingAmount: params.amountMaking,
            takingAmount: params.amountTaking,
            makerAsset: new Address(CONFIG.tokens.WETH),
            takerAsset: new Address(CONFIG.tokens.BTC)
        },
        {
            hashLock: HashLock.forSingleFill(params.hashLockSecret),

            timeLocks: TimeLocks.new({
                srcWithdrawal: 10n, // 10s finality lock for test
                srcPublicWithdrawal: 120n, // 2m for private withdrawal
                srcCancellation: 121n, // 1sec public withdrawal
                srcPublicCancellation: 122n, // 1sec private cancellation
                dstWithdrawal: 10n, // 10s finality lock for test
                dstPublicWithdrawal: 100n, // 100sec private withdrawal
                dstCancellation: 101n // 1sec public withdrawal
            }),
            srcChainId: CONFIG.chains.src,
            dstChainId: CONFIG.chains.dst,
            srcSafetyDeposit: CONFIG.deposits.src,
            dstSafetyDeposit: CONFIG.deposits.dst
        },
        {
            auction: new AuctionDetails({
                initialRateBump: 0,
                points: [],
                duration: 120n,
                startTime: params.startTime,
            }),
            whitelist: [{
                address: new Address(CONFIG.resolverAddress),
                allowFrom: 0n
            }],
            resolvingStartTime: 0n
        },
        {
            nonce: randBigInt(UINT_40_MAX),
            allowPartialFills: false,
            allowMultipleFills: false
        }
    )
}

