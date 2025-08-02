// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "forge-std/Script.sol";
import {EscrowFactory} from "../src/EscrowFactory.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Resolver} from "../src/Resolver.sol";
import {IEscrowFactory} from "../lib/cross-chain-swap/contracts/interfaces/IEscrowFactory.sol";
import {IOrderMixin} from "limit-order-protocol/contracts/interfaces/IOrderMixin.sol";

contract DeployTestEscrowFactory is Script {
    function run() external {
        address limitOrderProtocol = 0x111111125421cA6dc452d289314280a0f8842A65;
        address feeToken = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2; // WETH
        address owner = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
        address accessToken = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 ;// WETH as access token for simplicity
        uint32 rescueDelaySrc = uint32(60 * 60); // 1 hour
        uint32 rescueDelayDst = uint32(60 * 60); // 1 hour

        vm.startBroadcast();

        // 1. Deploy EscrowFactory
        EscrowFactory factory = new EscrowFactory(
            limitOrderProtocol,
            IERC20(feeToken),
            IERC20(accessToken),
            owner,
            rescueDelaySrc,
            rescueDelayDst
        );

        // 2. Deploy Resolver, passing factory address
        new Resolver(
            IEscrowFactory(factory),
            IOrderMixin(limitOrderProtocol),
            owner
        );

        vm.stopBroadcast();
    }
}
