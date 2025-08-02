import { parseEther } from "ethers"

export const CONFIG = {
    networkRPC: "http://127.0.0.1:8545",
    userPk: '0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97',
    resolverPk: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
    resolverAddress: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
    contracts: {
        escrowFactory: "0xE4166E7107f59917ae783d28c10c569fBac1bEA1",
        resolverContract: "0x1e34aaC4a7506Ff6F140158D6B9E62F845760435",
        limitOrderContract: "0x111111125421cA6dc452d289314280a0f8842A65",
        resolverBitcoinContract: "0x0000000000000000000000000000000000000000"
    },
    tokens: {
        USDC: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        WETH: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        BTC: "0x0000000000000000000000000000000000000000"
    },
    chains: { evm: 1, bitcoin: 0, src: 1, dst: 100 },
    amounts: { making: parseEther('0.1'), taking: parseEther('0.099') },
    deposits: { src: parseEther('0'), dst: parseEther('0') }
} as const

export const WETH_ABI = [
    "function deposit() external payable",
    "function approve(address spender, uint256 amount) external returns (bool)",
    "function balanceOf(address account) external view returns (uint256)",
    "function transfer(address recipient, uint256 amount) external returns (bool)"
]
