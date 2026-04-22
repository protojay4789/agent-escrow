// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title TECHPaymentRouter
 * @notice Dual-payment router for $TECH token — dynamically splits payments between burn and treasury.
 * @dev Burn ratio adapts to market conditions via off-chain keeper updates.
 *      Discount ensures $TECH payment is always cheaper than USDC equivalent.
 *      Inspired by GMX fee distribution + Velodrome vote-escrowed mechanics.
 */
contract TECHPaymentRouter is Ownable {
    using SafeERC20 for IERC20;

    // ─── Errors ────────────────────────────────────────────────────────────
    error ZeroAmount();
    error RatioOutOfBounds(uint256 provided, uint256 min, uint256 max);
    error DiscountTooHigh(uint256 provided, uint256 max);
    error NoChange();

    // ─── Events ────────────────────────────────────────────────────────────
    event PaymentProcessed(
        address indexed buyer,
        uint256 totalPaid,
        uint256 burned,
        uint256 recycled
    );
    event BurnRatioUpdated(uint256 oldRatio, uint256 newRatio, address indexed updater);
    event DiscountUpdated(uint256 oldDiscount, uint256 newDiscount, address indexed updater);
    event TreasuryUpdated(address oldTreasury, address newTreasury);

    // ─── Constants ─────────────────────────────────────────────────────────
    IERC20 public immutable techToken;
    address public constant BURN_ADDRESS = 0x000000000000000000000000000000000000dEaD;

    uint256 public constant MIN_BURN_BPS = 1000;  // 10%
    uint256 public constant MAX_BURN_BPS = 9000;  // 90%
    uint256 public constant MAX_DISCOUNT_BPS = 5000; // 50% max discount

    // ─── State ─────────────────────────────────────────────────────────────
    address public treasury;

    /// @notice Burn ratio in basis points. 5000 = 50% burn, 50% recycle.
    uint256 public burnRatioBps;

    /// @notice Discount off USDC price in basis points. 2500 = 25% cheaper than USDC.
    uint256 public discountBps;

    /// @notice Cumulative total burned (for dashboard/tracking).
    uint256 public totalBurned;

    /// @notice Cumulative total recycled to treasury.
    uint256 public totalRecycled;

    // ─── Constructor ───────────────────────────────────────────────────────
    constructor(
        address _techToken,
        address _treasury,
        uint256 _initialBurnRatio,
        uint256 _initialDiscount
    ) Ownable(msg.sender) {
        require(_techToken != address(0), "ZeroAddress");
        require(_treasury != address(0), "ZeroAddress");
        require(_initialBurnRatio >= MIN_BURN_BPS && _initialBurnRatio <= MAX_BURN_BPS, "RatioOOB");
        require(_initialDiscount <= MAX_DISCOUNT_BPS, "DiscountTooHigh");

        techToken = IERC20(_techToken);
        treasury = _treasury;
        burnRatioBps = _initialBurnRatio;
        discountBps = _initialDiscount;
    }

    // ─── Core ──────────────────────────────────────────────────────────────

    /**
     * @notice Process a $TECH payment — split between burn and treasury.
     * @param amount Total $TECH tokens paid by user.
     */
    function processPayment(uint256 amount) external {
        if (amount == 0) revert ZeroAmount();

        techToken.safeTransferFrom(msg.sender, address(this), amount);

        uint256 burnAmount = (amount * burnRatioBps) / 10_000;
        uint256 recycleAmount = amount - burnAmount;

        if (burnAmount > 0) {
            techToken.safeTransfer(BURN_ADDRESS, burnAmount);
            totalBurned += burnAmount;
        }
        if (recycleAmount > 0) {
            techToken.safeTransfer(treasury, recycleAmount);
            totalRecycled += recycleAmount;
        }

        emit PaymentProcessed(msg.sender, amount, burnAmount, recycleAmount);
    }

    /**
     * @notice Calculate how much $TECH is needed for a given USDC equivalent price.
     * @param usdcPrice The full price in USDC (e.g., 10e6 for $10 USDC with 6 decimals).
     * @param techPriceUsd Current $TECH price in USD (18 decimals).
     * @return techAmount Amount of $TECH needed (18 decimals).
     */
    function calculateTechAmount(
        uint256 usdcPrice,
        uint256 techPriceUsd
    ) public view returns (uint256 techAmount) {
        // discounted_usdc = usdcPrice * (10000 - discountBps) / 10000
        uint256 discountedUsdc = (usdcPrice * (10_000 - discountBps)) / 10_000;
        // techAmount = discountedUsdc / techPriceUsd
        if (techPriceUsd > 0) {
            techAmount = (discountedUsdc * 1e18) / techPriceUsd;
        }
    }

    // ─── Admin (keeper-callable) ───────────────────────────────────────────

    /**
     * @notice Update the burn/recycle ratio. Called by keeper bot.
     * @param newRatioBps New burn ratio (1000-9000).
     */
    function updateBurnRatio(uint256 newRatioBps) external onlyOwner {
        if (newRatioBps < MIN_BURN_BPS || newRatioBps > MAX_BURN_BPS) {
            revert RatioOutOfBounds(newRatioBps, MIN_BURN_BPS, MAX_BURN_BPS);
        }
        if (newRatioBps == burnRatioBps) revert NoChange();

        uint256 oldRatio = burnRatioBps;
        burnRatioBps = newRatioBps;
        emit BurnRatioUpdated(oldRatio, newRatioBps, msg.sender);
    }

    /**
     * @notice Update discount percentage.
     * @param newDiscountBps New discount in basis points (0-5000).
     */
    function updateDiscount(uint256 newDiscountBps) external onlyOwner {
        if (newDiscountBps > MAX_DISCOUNT_BPS) {
            revert DiscountTooHigh(newDiscountBps, MAX_DISCOUNT_BPS);
        }
        if (newDiscountBps == discountBps) revert NoChange();

        uint256 oldDiscount = discountBps;
        discountBps = newDiscountBps;
        emit DiscountUpdated(oldDiscount, newDiscountBps, msg.sender);
    }

    /**
     * @notice Update treasury address.
     */
    function updateTreasury(address newTreasury) external onlyOwner {
        require(newTreasury != address(0), "ZeroAddress");
        address oldTreasury = treasury;
        treasury = newTreasury;
        emit TreasuryUpdated(oldTreasury, newTreasury);
    }
}
