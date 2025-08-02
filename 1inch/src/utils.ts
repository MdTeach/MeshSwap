import { Contract, parseEther } from "ethers"
import { Wallet } from "./wallet"
import { WETH_ABI } from "./config"

export async function ensureWethAndApproval(
    wallet: Wallet,
    wethAddress: string,
    limitOrderContract: string
) {
    const wethContract = new Contract(wethAddress, WETH_ABI, wallet.signer)
    const ethAmount = parseEther("5");

    const userBal = await wethContract.balanceOf(await wallet.getAddress())
    if (userBal < ethAmount) {
        await wethContract.deposit({ value: ethAmount }).then(tx => tx.wait());
    }

    await wethContract.approve(limitOrderContract, ethAmount).then(tx => tx.wait());

}
