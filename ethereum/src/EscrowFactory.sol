// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.23;

import {EscrowFactory as BaseEscrowFactory} from "cross-chain-swap/EscrowFactory.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract EscrowFactory is BaseEscrowFactory {
    constructor(
        address limitOrderProtocol,
        IERC20 feeToken,
        IERC20 accessToken,
        address owner, uint32 rescueDelaySrc,
        uint32 rescueDelayDst
    ) BaseEscrowFactory(limitOrderProtocol, feeToken, accessToken, owner, rescueDelayDst, rescueDelayDst) {}
}
