// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/DIDRegistry.sol";

contract Deploy is Script {
    function run() external {
        vm.startBroadcast();
        DIDRegistry registry = new DIDRegistry();
        console.log("DIDRegistry deployed at:", address(registry));
        vm.stopBroadcast();
    }
}
