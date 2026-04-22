// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {AgentEscrow} from "../contracts/AgentEscrow.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// Mock USDC (6 decimals) for testing
contract MockUSDC {
    string public name = "USD Coin";
    string public symbol = "USDC";
    uint8 public decimals = 6;
    uint256 public totalSupply;

    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    function mint(address to, uint256 amount) external {
        balanceOf[to] += amount;
        totalSupply += amount;
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        return true;
    }

    function transfer(address to, uint256 amount) external returns (bool) {
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        allowance[from][msg.sender] -= amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        return true;
    }
}

// Helper to expose _hashTypedDataV4 for testing
contract AgentEscrowHarness is AgentEscrow {
    constructor(address _usdc, address _aiValidator) AgentEscrow(_usdc, _aiValidator) {}
    function hashTypedDataV4(bytes32 structHash) external view returns (bytes32) {
        return _hashTypedDataV4(structHash);
    }
}

contract AgentEscrowTest is Test {
    AgentEscrowHarness public escrow;
    MockUSDC public usdc;

    address public owner = address(this);
    address public buyer = address(0x1);
    address public seller = address(0x2);
    uint256 public aiValidatorPK = 0xA11CE; // Known private key
    address public aiValidator;
    address public stranger = address(0x4);

    uint256 constant AMOUNT = 100 * 1e6; // 100 USDC (6 decimals)

    function setUp() public {
        aiValidator = vm.addr(aiValidatorPK); // Derive address from private key
        usdc = new MockUSDC();
        escrow = new AgentEscrowHarness(address(usdc), aiValidator);

        // Fund buyer with USDC
        usdc.mint(buyer, 1000 * 1e6);

        // Buyer approves escrow contract
        vm.prank(buyer);
        usdc.approve(address(escrow), 1000 * 1e6);
    }

    // ─── Constructor Tests ─────────────────────────────────────────────────

    function test_constructorSetsCorrectValues() public view {
        assertEq(address(escrow.USDC()), address(usdc));
        assertEq(escrow.AI_VALIDATOR(), aiValidator);
        assertEq(escrow.nextEscrowId(), 0);
    }

    function test_constructorRevertsZeroUSDC() public {
        vm.expectRevert(AgentEscrow.ZeroAddress.selector);
        new AgentEscrow(address(0), aiValidator);
    }

    function test_constructorRevertsZeroValidator() public {
        vm.expectRevert(AgentEscrow.ZeroAddress.selector);
        new AgentEscrow(address(usdc), address(0));
    }

    // ─── Create Escrow Tests ───────────────────────────────────────────────

    function test_createEscrow() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        assertEq(id, 0);
        assertEq(escrow.nextEscrowId(), 1);

        (address b, address s, uint256 amount, , AgentEscrow.EscrowState state, ) = escrow.getEscrow(id);
        assertEq(b, buyer);
        assertEq(s, seller);
        assertEq(amount, AMOUNT);
        assertEq(uint8(state), uint8(AgentEscrow.EscrowState.Created));

        // USDC transferred to escrow
        assertEq(usdc.balanceOf(address(escrow)), AMOUNT);
        assertEq(usdc.balanceOf(buyer), 900 * 1e6);
    }

    function test_createEscrowEmitsEvent() public {
        vm.prank(buyer);
        vm.expectEmit(true, true, false, true);
        emit AgentEscrow.EscrowCreated(0, buyer, seller, AMOUNT);
        escrow.createEscrow(seller, AMOUNT);
    }

    function test_createEscrowRevertsZeroSeller() public {
        vm.prank(buyer);
        vm.expectRevert(AgentEscrow.ZeroAddress.selector);
        escrow.createEscrow(address(0), AMOUNT);
    }

    function test_createEscrowRevertsZeroAmount() public {
        vm.prank(buyer);
        vm.expectRevert(AgentEscrow.ZeroAmount.selector);
        escrow.createEscrow(seller, 0);
    }

    // ─── Mark Complete Tests ───────────────────────────────────────────────

    function test_markComplete() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.prank(seller);
        escrow.markComplete(id);

        (, , , , AgentEscrow.EscrowState state, ) = escrow.getEscrow(id);
        assertEq(uint8(state), uint8(AgentEscrow.EscrowState.Completed));
    }

    function test_markCompleteEmitsEvent() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.prank(seller);
        vm.expectEmit(true, true, false, true);
        emit AgentEscrow.EscrowCompleted(id, seller);
        escrow.markComplete(id);
    }

    function test_markCompleteRevertsNotSeller() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.prank(stranger);
        vm.expectRevert(AgentEscrow.OnlySeller.selector);
        escrow.markComplete(id);
    }

    function test_markCompleteRevertsAlreadyCompleted() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.prank(seller);
        escrow.markComplete(id);

        vm.prank(seller);
        vm.expectRevert(AgentEscrow.AlreadyCompleted.selector);
        escrow.markComplete(id);
    }

    // ─── Validate and Release Tests ────────────────────────────────────────

    function test_validateAndRelease() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.prank(seller);
        escrow.markComplete(id);

        // Sign validation with AI validator
        bytes memory signature = _signValidation(id);

        uint256 sellerBalanceBefore = usdc.balanceOf(seller);

        vm.prank(address(0x5)); // Anyone can call with valid signature
        escrow.validateAndRelease(id, signature);

        assertEq(usdc.balanceOf(seller), sellerBalanceBefore + AMOUNT);
        assertEq(usdc.balanceOf(address(escrow)), 0);

        (, , , , AgentEscrow.EscrowState state, ) = escrow.getEscrow(id);
        assertEq(uint8(state), uint8(AgentEscrow.EscrowState.Released));
    }

    function test_validateAndReleaseRevertsNotCompleted() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        bytes memory signature = _signValidation(id);

        vm.expectRevert(AgentEscrow.NotCompleted.selector);
        escrow.validateAndRelease(id, signature);
    }

    function test_validateAndReleaseRevertsInvalidSignature() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.prank(seller);
        escrow.markComplete(id);

        // Sign with wrong key — encode garbage as bytes signature
        bytes memory badSig = abi.encodePacked(bytes32(uint256(1)), bytes32(uint256(2)), uint8(27));
        vm.expectRevert(AgentEscrow.InvalidSignature.selector);
        escrow.validateAndRelease(id, badSig);
    }

    // ─── Refund Tests ──────────────────────────────────────────────────────

    function test_refund() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        // Warp past deadline
        vm.warp(block.timestamp + 8 days);

        uint256 buyerBalanceBefore = usdc.balanceOf(buyer);

        vm.prank(buyer);
        escrow.refund(id);

        assertEq(usdc.balanceOf(buyer), buyerBalanceBefore + AMOUNT);
        assertEq(usdc.balanceOf(address(escrow)), 0);

        (, , , , AgentEscrow.EscrowState state, ) = escrow.getEscrow(id);
        assertEq(uint8(state), uint8(AgentEscrow.EscrowState.Refunded));
    }

    function test_refundEmitsEvent() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.warp(block.timestamp + 8 days);

        vm.prank(buyer);
        vm.expectEmit(true, true, false, true);
        emit AgentEscrow.EscrowRefunded(id, buyer, AMOUNT);
        escrow.refund(id);
    }

    function test_refundRevertsNotBuyer() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.warp(block.timestamp + 8 days);

        vm.prank(stranger);
        vm.expectRevert(AgentEscrow.OnlyBuyer.selector);
        escrow.refund(id);
    }

    function test_refundRevertsBeforeDeadline() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.prank(buyer);
        vm.expectRevert(AgentEscrow.DeadlineExpired.selector);
        escrow.refund(id);
    }

    function test_refundRevertsIfCompleted() public {
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        vm.prank(seller);
        escrow.markComplete(id);

        vm.warp(block.timestamp + 8 days);

        vm.prank(buyer);
        vm.expectRevert(AgentEscrow.AlreadyCompleted.selector);
        escrow.refund(id);
    }

    // ─── Full Lifecycle Test ───────────────────────────────────────────────

    function test_fullLifecycle() public {
        // 1. Buyer creates escrow
        vm.prank(buyer);
        uint256 id = escrow.createEscrow(seller, AMOUNT);

        // 2. Seller marks work complete
        vm.prank(seller);
        escrow.markComplete(id);

        // 3. AI validator signs and releases
        bytes memory signature = _signValidation(id);
        escrow.validateAndRelease(id, signature);

        // 4. Verify final state
        assertEq(usdc.balanceOf(seller), AMOUNT);
        assertEq(usdc.balanceOf(address(escrow)), 0);
    }

    // ─── Helpers ───────────────────────────────────────────────────────────

    function _signValidation(uint256 escrowId) internal view returns (bytes memory) {
        bytes32 structHash = keccak256(abi.encode(
            keccak256("Validation(uint256 escrowId)"),
            escrowId
        ));
        bytes32 digest = escrow.hashTypedDataV4(structHash);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(aiValidatorPK, digest);
        return abi.encodePacked(r, s, v);
    }
}
