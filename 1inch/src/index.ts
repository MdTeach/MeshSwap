import { uint8ArrayToHex, UINT_40_MAX } from "@1inch/byte-utils"
import { CrossChainOrder, Address, HashLock, TimeLocks, AuctionDetails, randBigInt } from '@1inch/cross-chain-sdk'
import { JsonRpcProvider, parseEther, parseUnits, randomBytes } from "ethers"
import { Wallet } from "./wallet"

const CONFIG = {
    userPk: '0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97',
    resolverPk: '0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356',
    contracts: {
        escrowFactory: "0x1e34aaC4a7506Ff6F140158D6B9E62F845760435",
        limitOrderContract: "0xE4166E7107f59917ae783d28c10c569fBac1bEA1",
    },
    assets: {
        maker: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        taker: "0x8965349fb649a33a30cbfda057d8ec2c48abe2a2"
    },
    chains: { src: 1, dst: 100 },
    amounts: { making: parseUnits('100', 6), taking: parseUnits('99', 6) },
    deposits: { src: parseEther('0.001'), dst: parseEther('0.001') }
} as const

async function createCrossChainOrder() {
    const provider = new JsonRpcProvider("http://127.0.0.1:8545", CONFIG.chains.src, {
        cacheTimeout: -1,
        staticNetwork: true
    })

    const [srcChainUser, resolverWallet] = [
        new Wallet(CONFIG.userPk, provider),
        new Wallet(CONFIG.resolverPk, provider)
    ]

    const [secret, srcTimestamp] = [
        uint8ArrayToHex(randomBytes(32)),
        BigInt((await provider.getBlock('latest'))!.timestamp)
    ]

    const order = CrossChainOrder.new(
        new Address(CONFIG.contracts.escrowFactory),
        {
            salt: randBigInt(1000n),
            maker: new Address(await srcChainUser.getAddress()),
            makingAmount: CONFIG.amounts.making,
            takingAmount: CONFIG.amounts.taking,
            makerAsset: new Address(CONFIG.assets.maker),
            takerAsset: new Address(CONFIG.assets.taker)
        },
        {
            hashLock: HashLock.forSingleFill(secret),
            timeLocks: TimeLocks.new({
                srcWithdrawal: 10n,
                srcPublicWithdrawal: 120n,
                srcCancellation: 121n,
                srcPublicCancellation: 122n,
                dstWithdrawal: 10n,
                dstPublicWithdrawal: 100n,
                dstCancellation: 101n
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
                startTime: srcTimestamp
            }),
            whitelist: [{
                address: new Address(await resolverWallet.getAddress()),
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

    const signature = await srcChainUser.signOrder(CONFIG.chains.src, order)
    console.log(`Order Hash: ${order.getOrderHash(CONFIG.chains.src)}`)
}

createCrossChainOrder().catch(console.error)