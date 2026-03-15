// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/DIDRegistry.sol";

contract DIDRegistryTest is Test {
    DIDRegistry registry;
    address alice = address(0x1);
    address bob   = address(0x2);

    function setUp() public {
        registry = new DIDRegistry();
    }

    function testRegisterDID() public {
        vm.prank(alice);
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
        DIDRegistry.DIDRecord memory r = registry.resolve("did:key:z6MkAlice");
        assertEq(r.controller, alice);
        assertTrue(r.active);
    }

    function testCannotRegisterDuplicate() public {
        vm.startPrank(alice);
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
        vm.expectRevert("DID already registered");
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
        vm.stopPrank();
    }

    function testInvalidPublicKeyLength() public {
        vm.prank(alice);
        vm.expectRevert("Invalid Ed25519 public key length");
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
    }

    function testUpdateDID() public {
        vm.startPrank(alice);
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
        registry.update("did:key:z6MkAlice", "https://example.com/alice-v2.json", keccak256("alice-doc-v2"));
        DIDRegistry.DIDRecord memory r = registry.resolve("did:key:z6MkAlice");
        assertEq(r.nonce, 1);
        vm.stopPrank();
    }

    function testOnlyControllerCanUpdate() public {
        vm.prank(alice);
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
        vm.prank(bob);
        vm.expectRevert("Not the DID controller");
        registry.update("did:key:z6MkAlice", "https://example.com/bob.json", keccak256("bob-doc"));
    }

    function testDeactivateDID() public {
        vm.startPrank(alice);
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
        registry.deactivate("did:key:z6MkAlice");
        assertFalse(registry.isActive("did:key:z6MkAlice"));
        vm.stopPrank();
    }

    function testCannotUpdateDeactivated() public {
        vm.startPrank(alice);
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
        registry.deactivate("did:key:z6MkAlice");
        vm.expectRevert("DID is deactivated");
        registry.update("did:key:z6MkAlice", "https://example.com/alice-v2.json", keccak256("alice-doc-v2"));
        vm.stopPrank();
    }

    function testResolveUnregistered() public {
        vm.expectRevert("DID not found");
        registry.resolve("did:key:z6MkNotExist");
    }

    function testIsActive() public {
        assertFalse(registry.isActive("did:key:z6MkNotExist"));
        vm.prank(alice);
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
        assertTrue(registry.isActive("did:key:z6MkAlice"));
    }

    function testGetDIDsByController() public {
        vm.startPrank(alice);
        registry.register(
            "did:key:z6MkAlice1",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice1.json",
            keccak256("alice1-doc")
        );
        registry.register(
            "did:key:z6MkAlice2",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice2.json",
            keccak256("alice2-doc")
        );
        string[] memory dids = registry.getDIDsByController(alice);
        assertEq(dids.length, 2);
        vm.stopPrank();
    }

    function testEventsEmitted() public {
        vm.prank(alice);
        vm.expectEmit(true, true, false, true);
        emit DIDRegistry.DIDRegistered("did:key:z6MkAlice", alice, block.timestamp);
        registry.register(
            "did:key:z6MkAlice",
            bytes(hex"0102030405060708091011121314151617181920212223242526272829303132"),
            "https://example.com/alice.json",
            keccak256("alice-doc")
        );
    }
}
