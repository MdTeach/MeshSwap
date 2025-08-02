import { uint8ArrayToHex, UINT_40_MAX } from "@1inch/byte-utils"
import { TakerTraits, CrossChainOrder, AmountMode, Address, HashLock, TimeLocks, AuctionDetails, randBigInt, EscrowFactory as EF } from '@1inch/cross-chain-sdk'
import { EscrowFactory } from './escrow-factory'
import { JsonRpcProvider, parseEther, parseUnits, randomBytes, Contract } from "ethers"
import { Wallet } from "./wallet"
import { Resolver } from "./resolver"

const CONFIG = {
    userPk: '0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97',
    resolverPk: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
    contracts: {
        escrowFactory: "0xE4166E7107f59917ae783d28c10c569fBac1bEA1",
        resolverContract: "0x1e34aaC4a7506Ff6F140158D6B9E62F845760435",
        limitOrderContract: "0x111111125421cA6dc452d289314280a0f8842A65",
    },
    tokens: {
        USDC: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        WETH: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    },
    chains: { src: 1, dst: 100 },
    amounts: { making: parseEther('0.1'), taking: parseEther('0.099') },
    deposits: { src: parseEther('0'), dst: parseEther('0') }
} as const

const WETH_ABI = [
    "function deposit() external payable",
    "function approve(address spender, uint256 amount) external returns (bool)",
    "function balanceOf(address account) external view returns (uint256)",
    "function transfer(address recipient, uint256 amount) external returns (bool)"
]





async function ensureWethAndApproval(
    wallet: Wallet,
    ethAmount: bigint,
    wethAddress: string,
    limitOrderContract: string
) {
    const wethContract = new Contract(wethAddress, WETH_ABI, wallet.signer)

    if (ethAmount > 0n) {
        const depositTx = await wethContract.deposit({ value: ethAmount * 2n }) // double the amount to ensure enough for gas
        await depositTx.wait()
    }

    const wethBalance = await wethContract.balanceOf(await wallet.getAddress())
    if (wethBalance > 0n) {
        const approveTx = await wethContract.approve(limitOrderContract, wethBalance)
        await approveTx.wait()
    }

    // send some WETH to resolver contract for approval
    const amountInWei = parseEther("0.1"); // if you want to send 1 ETH worth
    const transfer_tx = await wethContract.transfer(CONFIG.contracts.resolverContract, amountInWei);
    await transfer_tx.wait();
}

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

    console.log("Using the src timestamp:", srcTimestamp);


    const srcChainResolver = new Wallet(CONFIG.resolverPk, provider)

    await ensureWethAndApproval(
        srcChainUser,
        parseEther('0.1'),
        CONFIG.tokens.WETH,
        CONFIG.contracts.limitOrderContract
    )

    const order = CrossChainOrder.new(
        new Address(CONFIG.contracts.escrowFactory),
        {
            salt: randBigInt(1000n),
            maker: new Address(await srcChainUser.getAddress()),
            makingAmount: CONFIG.amounts.making,
            takingAmount: CONFIG.amounts.taking,
            makerAsset: new Address(CONFIG.tokens.WETH),
            takerAsset: new Address(CONFIG.tokens.USDC)
        },
        {
            hashLock: HashLock.forSingleFill(secret),

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

    const srcChainId = CONFIG.chains.src;
    const orderHash = order.getOrderHash(srcChainId)
    const signature = await srcChainUser.signOrder(CONFIG.chains.src, order)
    console.log(`Order Hash: ${order.getOrderHash(CONFIG.chains.src)}`)

    // Resolver fills the order
    const resolverContract = new Resolver(CONFIG.contracts.resolverContract, CONFIG.contracts.resolverContract)
    const fillAmount = order.makingAmount;
    const txn_data = resolverContract.deploySrc(
        srcChainId,
        order,
        signature,
        TakerTraits.default()
            .setExtension(order.extension)
            .setAmountMode(AmountMode.maker)
            .setAmountThreshold(order.takingAmount),
        fillAmount
    );
    const { txHash: orderFillHash, blockHash: srcDeployBlock } = await srcChainResolver.send_new(
        txn_data
    )
    console.log(`Order Fill Tx Hash: ${orderFillHash} and block has ${srcDeployBlock}`);
    console.log(`[${srcChainId}]`, `Order ${orderHash} filled for ${fillAmount} in tx ${orderFillHash}`)

    const escrowFactory = new EscrowFactory(provider, CONFIG.contracts.escrowFactory)
    const srcEscrowEvent = await escrowFactory.getSrcDeployEvent(srcDeployBlock)
    console.log("Abishek got the srcEscrowEvent ", srcEscrowEvent);
    

    
    const ESCROW_SRC_IMPLEMENTATION = await escrowFactory.getSourceImpl()
    const srcEscrowAddress = new EF(new Address(CONFIG.contracts.escrowFactory)).getSrcEscrowAddress(
        srcEscrowEvent[0],
        ESCROW_SRC_IMPLEMENTATION
    )
    console.log(`Source Escrow Address: ${srcEscrowAddress.toString()}`)


    // Sleep for 10 seconds to simulate the time locks
    await new Promise(resolve => setTimeout(resolve, 10000));

    console.log(`[{srcChainId}]`, `Withdrawing funds for resolver from ${srcEscrowAddress}`)
    const txn =
        resolverContract.withdraw('src', srcEscrowAddress, secret, srcEscrowEvent[0]);
    console.log("got the txn ", txn);

    const { txHash: resolverWithdrawHash } = await srcChainResolver.send(
        txn
    )
    console.log(
        `[${srcChainId}]`,
        `Withdrew funds for resolver from ${srcEscrowAddress} to ${ await srcChainResolver.getAddress()} in tx ${resolverWithdrawHash}`
    )

    // Qurey the balance of WETH of the resolver contract
    const wethContract = new Contract(CONFIG.tokens.WETH, WETH_ABI, provider)
    const resolverWethBalance = await wethContract.balanceOf(await srcChainResolver.getAddress())
    console.log(`Resolver WETH Balance: ${resolverWethBalance.toString()}`)
    



}

createCrossChainOrder().catch(console.error)