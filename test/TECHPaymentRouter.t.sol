// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {TECHPaymentRouter} from "../contracts/TECHPaymentRouter.sol";

/// @dev Mock ERC20 for testing
contract MockTECH is ERC20 {
    constructor() ERC20("GenTech", "TECH") {}

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract TECHPaymentRouterTest is Test {
    TECHPaymentRouter public router;
    MockTECH public token;

    address public owner = makeAddr("owner");
    address public treasury = makeAddr("treasury");
    address public alice = makeAddr("alice");
    address public constant BURN = 0x000000000000000000000000000000000000dEaD;

    uint256 constant INITIAL_BURN = 5000; // 50%
    uint256 constant INITIAL_DISCOUNT = 2500; // 25%

    function setUp() public {
        vm.startPrank(owner);
        token = new MockTECH();
        router = new TECHPaymentRouter(
            address(token),
            treasury,
            INITIAL_BURN,
            INITIAL_DISCOUNT
        );
        vm.stopPrank();

        // Mint tokens to alice for testing
        token.mint(alice, 1_000_000e18);
        // Approve router
        vm.prank(alice);
        token.approve(address(router), type(uint256).max);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CORE: Payment Processing
    // ═══════════════════════════════════════════════════════════════════════

    function test_processPayment_basic50split() public {
        uint256 payment = 1000e18;

        vm.prank(alice);
        router.processPayment(payment);

        // 50% burned, 50% to treasury
        assertEq(token.balanceOf(BURN), 500e18);
        assertEq(token.balanceOf(treasury), 500e18);
        assertEq(token.balanceOf(alice), 1_000_000e18 - payment);
        assertEq(router.totalBurned(), 500e18);
        assertEq(router.totalRecycled(), 500e18);
    }

    function test_processPayment_70_30split() public {
        // Set 70% burn
        vm.prank(owner);
        router.updateBurnRatio(7000);

        uint256 payment = 1000e18;
        vm.prank(alice);
        router.processPayment(payment);

        assertEq(token.balanceOf(BURN), 700e18);
        assertEq(token.balanceOf(treasury), 300e18);
    }

    function test_processPayment_90_10split() public {
        // Max burn
        vm.prank(owner);
        router.updateBurnRatio(9000);

        uint256 payment = 1000e18;
        vm.prank(alice);
        router.processPayment(payment);

        assertEq(token.balanceOf(BURN), 900e18);
        assertEq(token.balanceOf(treasury), 100e18);
    }

    function test_processPayment_10_90split() public {
        // Min burn
        vm.prank(owner);
        router.updateBurnRatio(1000);

        uint256 payment = 1000e18;
        vm.prank(alice);
        router.processPayment(payment);

        assertEq(token.balanceOf(BURN), 100e18);
        assertEq(token.balanceOf(treasury), 900e18);
    }

    function test_processPayment_revertsOnZero() public {
        vm.prank(alice);
        vm.expectRevert(TECHPaymentRouter.ZeroAmount.selector);
        router.processPayment(0);
    }

    function test_processPayment_emitsEvent() public {
        uint256 payment = 1000e18;

        vm.expectEmit(true, true, false, true);
        emit TECHPaymentRouter.PaymentProcessed(alice, payment, 500e18, 500e18);

        vm.prank(alice);
        router.processPayment(payment);
    }

    function test_processPayment_multiplePaymentsAccumulate() public {
        vm.startPrank(alice);

        router.processPayment(1000e18);
        router.processPayment(2000e18);
        router.processPayment(500e18);

        vm.stopPrank();

        // Total: 3500e18, 50% burn = 1750e18
        assertEq(router.totalBurned(), 1750e18);
        assertEq(router.totalRecycled(), 1750e18);
        assertEq(token.balanceOf(BURN), 1750e18);
        assertEq(token.balanceOf(treasury), 1750e18);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DYNAMIC: Burn Ratio Updates
    // ═══════════════════════════════════════════════════════════════════════

    function test_updateBurnRatio_increasesTo70() public {
        vm.prank(owner);
        router.updateBurnRatio(7000);

        assertEq(router.burnRatioBps(), 7000);
    }

    function test_updateBurnRatio_revertsOutOfBounds() public {
        vm.prank(owner);

        // Below min
        vm.expectRevert();
        router.updateBurnRatio(999);

        // Above max
        vm.expectRevert();
        router.updateBurnRatio(9001);
    }

    function test_updateBurnRatio_revertsNoChange() public {
        vm.prank(owner);
        vm.expectRevert(TECHPaymentRouter.NoChange.selector);
        router.updateBurnRatio(5000); // same as initial
    }

    function test_updateBurnRatio_onlyOwner() public {
        vm.prank(alice);
        vm.expectRevert();
        router.updateBurnRatio(7000);
    }

    function test_updateBurnRatio_emitsEvent() public {
        vm.expectEmit(true, true, false, true);
        emit TECHPaymentRouter.BurnRatioUpdated(5000, 7000, owner);

        vm.prank(owner);
        router.updateBurnRatio(7000);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DYNAMIC: Discount Updates
    // ═══════════════════════════════════════════════════════════════════════

    function test_updateDiscount_tightenTo15() public {
        vm.prank(owner);
        router.updateDiscount(1500);

        assertEq(router.discountBps(), 1500);
    }

    function test_updateDiscount_revertsTooHigh() public {
        vm.prank(owner);
        vm.expectRevert();
        router.updateDiscount(5001);
    }

    function test_updateDiscount_revertsNoChange() public {
        vm.prank(owner);
        vm.expectRevert(TECHPaymentRouter.NoChange.selector);
        router.updateDiscount(2500); // same as initial
    }

    function test_updateDiscount_onlyOwner() public {
        vm.prank(alice);
        vm.expectRevert();
        router.updateDiscount(1500);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DYNAMIC: Price Calculation
    // ═══════════════════════════════════════════════════════════════════════

    function test_calculateTechAmount_basic() public {
        // $10 USDC price, $TECH at $0.10, 25% discount
        // discounted = $7.50 USDC (6 dec) = 7_500_000
        // tech needed = 7_500_000 * 1e18 / 0.1e18 = 75e6 (raw, mixed decimal)
        uint256 usdcPrice = 10e6;     // $10 USDC (6 decimals)
        uint256 techPrice = 0.1e18;   // $0.10 (18 decimals)

        uint256 result = router.calculateTechAmount(usdcPrice, techPrice);
        // result is in raw units: (usdc6 * 1e18) / tech18
        // = (7_500_000 * 1e18) / 1e17 = 75_000_000
        assertEq(result, 75_000_000);
    }

    function test_calculateTechAmount_noDiscount() public {
        vm.prank(owner);
        router.updateDiscount(0);

        // $10 USDC, $TECH at $0.10, 0% discount
        // = (10e6 * 1e18) / 0.1e18 = 100_000_000
        uint256 result = router.calculateTechAmount(10e6, 0.1e18);
        assertEq(result, 100_000_000);
    }

    function test_calculateTechAmount_maxDiscount() public {
        vm.prank(owner);
        router.updateDiscount(5000);

        // $10 USDC, $TECH at $0.10, 50% discount
        // = (5e6 * 1e18) / 0.1e18 = 50_000_000
        uint256 result = router.calculateTechAmount(10e6, 0.1e18);
        assertEq(result, 50_000_000);
    }

    function test_calculateTechAmount_highTokenPrice() public {
        // $10 USDC, $TECH at $1.00, 25% discount
        // = (7_500_000 * 1e18) / 1e18 = 7_500_000
        uint256 result = router.calculateTechAmount(10e6, 1e18);
        assertEq(result, 7_500_000);
    }

    function test_calculateTechAmount_zeroTokenPrice() public {
        uint256 result = router.calculateTechAmount(10e6, 0);
        assertEq(result, 0);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DYNAMIC: Keeper Simulation
    // ═══════════════════════════════════════════════════════════════════════

    function test_keeperSimulation_bullMarket() public {
        // Simulate: price pumping → keeper tightens discount, increases burn
        vm.startPrank(owner);
        router.updateDiscount(1500);   // Tighten discount from 25% → 15%
        router.updateBurnRatio(7000);  // More burn (70%)
        vm.stopPrank();

        // User pays 1000 $TECH
        vm.prank(alice);
        router.processPayment(1000e18);

        // 70% burned, 30% recycled
        assertEq(token.balanceOf(BURN), 700e18);
        assertEq(token.balanceOf(treasury), 300e18);

        // Discount is tighter (15%)
        assertEq(router.discountBps(), 1500);
    }

    function test_keeperSimulation_bearMarket() public {
        // Simulate: price dumping → keeper widens discount, less burn (more recycle to fund ecosystem)
        vm.startPrank(owner);
        router.updateDiscount(3500);   // Widen discount 25% → 35%
        router.updateBurnRatio(3000);  // Less burn (30%), more to treasury for competitions
        vm.stopPrank();

        vm.prank(alice);
        router.processPayment(1000e18);

        // 30% burned, 70% recycled to fund competitions/grants
        assertEq(token.balanceOf(BURN), 300e18);
        assertEq(token.balanceOf(treasury), 700e18);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════

    function test_treasuryUpdate() public {
        address newTreasury = makeAddr("newTreasury");

        vm.prank(owner);
        router.updateTreasury(newTreasury);

        assertEq(router.treasury(), newTreasury);

        // Payment goes to new treasury
        vm.prank(alice);
        router.processPayment(1000e18);

        assertEq(token.balanceOf(newTreasury), 500e18);
        assertEq(token.balanceOf(treasury), 0);
    }

    function test_treasuryUpdate_revertsOnZero() public {
        vm.prank(owner);
        vm.expectRevert("ZeroAddress");
        router.updateTreasury(address(0));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FUZZ TESTS
    // ═══════════════════════════════════════════════════════════════════════

    function testFuzz_processPayment_alwaysSplitsCorrectly(uint256 amount) public {
        // Bound to reasonable range
        amount = bound(amount, 1, 1_000_000e18);

        uint256 balanceBefore = token.balanceOf(alice);

        vm.prank(alice);
        router.processPayment(amount);

        // All tokens accounted for
        assertEq(
            token.balanceOf(BURN) + token.balanceOf(treasury) + token.balanceOf(alice),
            balanceBefore
        );
    }

    function testFuzz_updateBurnRatio_acceptsValidRange(uint256 ratio) public {
        ratio = bound(ratio, 1000, 9000);

        // Skip if it's the current value (would revert on NoChange)
        if (ratio == router.burnRatioBps()) {
            return;
        }

        vm.prank(owner);
        router.updateBurnRatio(ratio);

        assertEq(router.burnRatioBps(), ratio);
    }

    function testFuzz_updateDiscount_acceptsValidRange(uint256 discount) public {
        // Set to different value first so NoChange doesn't revert
        vm.prank(owner);
        router.updateDiscount(0);

        discount = bound(discount, 0, 5000);

        if (discount == 0) {
            // Already at 0, skip
            return;
        }

        vm.prank(owner);
        router.updateDiscount(discount);

        assertEq(router.discountBps(), discount);
    }

    function testFuzz_techAmount_alwaysLessThanUSDC(uint256 usdcPrice, uint256 techPrice) public {
        usdcPrice = bound(usdcPrice, 1e6, 1_000_000e6);  // $0.01 to $1M
        techPrice = bound(techPrice, 0.001e18, 1000e18);  // $0.001 to $1000

        uint256 techAmount = router.calculateTechAmount(usdcPrice, techPrice);

        if (techPrice > 0 && techAmount > 0) {
            // The $TECH cost should always be less than USDC price (because of discount)
            // techCost = techAmount * techPrice / 1e18
            uint256 techCost = (techAmount * techPrice) / 1e18;
            uint256 discountedUsdc = (usdcPrice * (10_000 - router.discountBps())) / 10_000;

            // Allow small rounding error
            assertLe(techCost, discountedUsdc + 1);
        }
    }
}
