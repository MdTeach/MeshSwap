import { Interface, Signature, TransactionRequest } from 'ethers'
import { Address, CrossChainOrder, TakerTraits, Immutables } from '@1inch/cross-chain-sdk'
import Contract from '../../ethereum/out/Resolver.sol/Resolver.json'

export class Resolver {
    private readonly iface = new Interface(Contract.abi)

    constructor(
        public readonly srcAddress: string,
        public readonly dstAddress: string
    ) { }

    public deploySrc(
        chainId: number,
        order: CrossChainOrder,
        signature: string,
        takerTraits: TakerTraits,
        amount: bigint,
        hashLock = order.escrowExtension.hashLockInfo
    ): TransactionRequest {
        const { r, yParityAndS: vs } = Signature.from(signature)
        const { args, trait } = takerTraits.encode()
        const immutables = order.toSrcImmutables(chainId, new Address(this.srcAddress), amount, hashLock)

        return {
            to: this.srcAddress,
            data: this.iface.encodeFunctionData('deploySrc', [
                immutables.build(),
                order.build(),
                r,
                vs,
                amount,
                trait,
                args
            ]),
            value: order.escrowExtension.srcSafetyDeposit
        }
    }

    public deployDst(
        /**
         * Immutables from SrcEscrowCreated event with complement applied
         */
        immutables: Immutables
    ): TransactionRequest {
        return {
            to: this.dstAddress,
            data: this.iface.encodeFunctionData('deployDst', [
                immutables.build(),
                immutables.timeLocks.toSrcTimeLocks().privateCancellation
            ]),
            value: immutables.safetyDeposit
        }
    }

    public withdraw(
        side: 'src' | 'dst',
        escrow: Address,
        secret: string,
        immutables: Immutables
    ): TransactionRequest {
        return {
            to: side === 'src' ? this.srcAddress : this.dstAddress,
            data: this.iface.encodeFunctionData('withdraw', [escrow.toString(), secret, immutables.build()])
        }
    }

    public cancel(side: 'src' | 'dst', escrow: Address, immutables: Immutables): TransactionRequest {
        return {
            to: side === 'src' ? this.srcAddress : this.dstAddress,
            data: this.iface.encodeFunctionData('cancel', [escrow.toString(), immutables.build()])
        }
    }
}
