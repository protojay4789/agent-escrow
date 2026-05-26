// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ServiceRegistry} from "../contracts/ServiceRegistry.sol";

contract ServiceRegistryTest is Test {
    ServiceRegistry public registry;

    address public owner = makeAddr("owner");
    address public provider = makeAddr("provider");
    address public agent = makeAddr("agent");
    address public agent2 = makeAddr("agent2");

    function setUp() public {
        vm.prank(owner);
        registry = new ServiceRegistry();
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PUBLISHING
    // ═══════════════════════════════════════════════════════════════════════

    function test_publishService() public {
        string[] memory caps = new string[](2);
        caps[0] = "parsing";
        caps[1] = "classification";

        vm.prank(provider);
        uint256 id = registry.publishService(
            "Data Parser",
            "Parses CSV and JSON data",
            10e6,  // $10 USDC
            caps
        );

        assertEq(id, 0);

        (uint256 serviceId, address serviceProvider, string memory name, , uint256 price, , bool active, , , ) =
            registry.getService(0);

        assertEq(serviceId, 0);
        assertEq(serviceProvider, provider);
        assertEq(name, "Data Parser");
        assertEq(price, 10e6);
        assertTrue(active);
    }

    function test_publishService_incrementsId() public {
        string[] memory caps = new string[](1);
        caps[0] = "test";

        vm.prank(provider);
        registry.publishService("Service 1", "Desc 1", 1e6, caps);

        vm.prank(provider);
        uint256 id2 = registry.publishService("Service 2", "Desc 2", 2e6, caps);

        assertEq(id2, 1);
        assertEq(registry.totalServices(), 2);
    }

    function test_publishService_revertsOnEmptyName() public {
        string[] memory caps = new string[](0);

        vm.prank(provider);
        vm.expectRevert("Name required");
        registry.publishService("", "Desc", 1e6, caps);
    }

    function test_publishService_emitsEvent() public {
        string[] memory caps = new string[](1);
        caps[0] = "parsing";

        vm.expectEmit();
        emit ServiceRegistry.ServicePublished(0, provider, "Data Parser", 10e6);

        vm.prank(provider);
        registry.publishService("Data Parser", "Desc", 10e6, caps);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DEACTIVATION
    // ═══════════════════════════════════════════════════════════════════════

    function test_deactivateService() public {
        string[] memory caps = new string[](1);
        caps[0] = "test";

        vm.prank(provider);
        uint256 id = registry.publishService("Service", "Desc", 1e6, caps);

        vm.prank(provider);
        registry.deactivateService(id);

        (, , , , , , bool active, , , ) = registry.getService(id);
        assertFalse(active);
    }

    function test_deactivateService_revertsIfNotProvider() public {
        string[] memory caps = new string[](1);
        caps[0] = "test";

        vm.prank(provider);
        uint256 id = registry.publishService("Service", "Desc", 1e6, caps);

        vm.prank(agent);
        vm.expectRevert("Not provider");
        registry.deactivateService(id);
    }

    function test_reactivateService() public {
        string[] memory caps = new string[](1);
        caps[0] = "test";

        vm.prank(provider);
        uint256 id = registry.publishService("Service", "Desc", 1e6, caps);

        vm.prank(provider);
        registry.deactivateService(id);

        vm.prank(provider);
        registry.reactivateService(id);

        (, , , , , , bool active, , , ) = registry.getService(id);
        assertTrue(active);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXECUTION RECORDING
    // ═══════════════════════════════════════════════════════════════════════

    function test_recordExecution() public {
        string[] memory caps = new string[](1);
        caps[0] = "parsing";

        vm.prank(provider);
        uint256 id = registry.publishService("Service", "Desc", 1e6, caps);

        vm.prank(provider);
        registry.recordExecution(id, agent, 1e6, true, "QmHash123");

        assertEq(registry.totalExecutions(), 1);
        assertEq(registry.getExecutionCount(id), 1);

        (address execAgent, uint256 paid, bool success, , string memory hash) =
            registry.getExecution(id, 0);

        assertEq(execAgent, agent);
        assertEq(paid, 1e6);
        assertTrue(success);
        assertEq(hash, "QmHash123");
    }

    function test_recordExecution_incrementsServiceStats() public {
        string[] memory caps = new string[](1);
        caps[0] = "test";

        vm.prank(provider);
        uint256 id = registry.publishService("Service", "Desc", 1e6, caps);

        vm.prank(provider);
        registry.recordExecution(id, agent, 1e6, true, "hash1");
        registry.recordExecution(id, agent2, 2e6, true, "hash2");

        (, , , , , , , , uint256 totalExec, uint256 totalRev) = registry.getService(id);
        assertEq(totalExec, 2);
        assertEq(totalRev, 3e6);
    }

    function test_recordExecution_revertsIfServiceInactive() public {
        string[] memory caps = new string[](1);
        caps[0] = "test";

        vm.prank(provider);
        uint256 id = registry.publishService("Service", "Desc", 1e6, caps);

        vm.prank(provider);
        registry.deactivateService(id);

        vm.prank(provider);
        vm.expectRevert("Service not active");
        registry.recordExecution(id, agent, 1e6, true, "hash");
    }

    function test_recordExecution_revertsOnZeroAgent() public {
        string[] memory caps = new string[](1);
        caps[0] = "test";

        vm.prank(provider);
        uint256 id = registry.publishService("Service", "Desc", 1e6, caps);

        vm.prank(provider);
        vm.expectRevert("Zero address");
        registry.recordExecution(id, address(0), 1e6, true, "hash");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CAPABILITY CHECK
    // ═══════════════════════════════════════════════════════════════════════

    function test_hasCapability() public {
        string[] memory caps = new string[](2);
        caps[0] = "parsing";
        caps[1] = "classification";

        vm.prank(provider);
        uint256 id = registry.publishService("Service", "Desc", 1e6, caps);

        assertTrue(registry.hasCapability(id, "parsing"));
        assertTrue(registry.hasCapability(id, "classification"));
        assertFalse(registry.hasCapability(id, "translation"));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // VIEW FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════

    function test_getService_empty() public {
        (, address serviceProvider, , , , , bool active, , , ) = registry.getService(0);
        assertEq(serviceProvider, address(0));
        assertFalse(active);
    }

    function test_totalServices() public {
        assertEq(registry.totalServices(), 0);

        string[] memory caps = new string[](1);
        caps[0] = "test";

        vm.prank(provider);
        registry.publishService("S1", "D1", 1e6, caps);

        assertEq(registry.totalServices(), 1);

        vm.prank(provider);
        registry.publishService("S2", "D2", 2e6, caps);

        assertEq(registry.totalServices(), 2);
    }
}
