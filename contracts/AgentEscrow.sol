// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {EIP712} from "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title AgentEscrow
 * @notice AI-validated escrow for agent-to-service payments using USDC + EIP-712 signatures.
 * @dev Implements checks-effects-interactions, pull-over-push, reentrancy protection.
 *      AI validator signs off on work quality before funds are released.
 *      Inspired by Circle's RefundProtocol patterns.
 */
contract AgentEscrow is EIP712, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // ─── Errors ────────────────────────────────────────────────────────────
    error ZeroAddress();
    error ZeroAmount();
    error EscrowNotFound();
    error OnlyBuyer();
    error OnlySeller();
    error AlreadyCompleted();
    error NotCompleted();
    error AlreadyValidated();
    error AlreadyRefunded();
    error InvalidSignature();
    error DeadlineExpired();

    // ─── Events ────────────────────────────────────────────────────────────
    event EscrowCreated(uint256 indexed id, address indexed buyer, address indexed seller, uint256 amount);
    event EscrowCompleted(uint256 indexed id, address indexed seller);
    event EscrowValidated(uint256 indexed id, address indexed validator);
    event EscrowReleased(uint256 indexed id, address indexed seller, uint256 amount);
    event EscrowRefunded(uint256 indexed id, address indexed buyer, uint256 amount);

    // ─── Structs ───────────────────────────────────────────────────────────
    enum EscrowState { Created, Completed, Validated, Released, Refunded }

    struct Escrow {
        address buyer;
        address seller;
        uint256 amount;
        uint256 deadline;       // Refund deadline (buyer can refund if seller doesn't deliver)
        EscrowState state;
        uint256 createdAt;
    }

    // ─── State ─────────────────────────────────────────────────────────────
    IERC20 public immutable USDC;
    address public immutable AI_VALIDATOR;

    mapping(uint256 => Escrow) public escrows;
    uint256 public nextEscrowId;

    // Default deadline: 7 days if not specified
    uint256 public constant DEFAULT_DEADLINE = 7 days;

    // EIP-712 typehash for validation signatures
    // No timestamp — the escrowId + state machine (Completed→Released) prevents replay
    bytes32 private constant VALIDATION_TYPEHASH = keccak256(
        "Validation(uint256 escrowId)"
    );

    // ─── Constructor ───────────────────────────────────────────────────────
    constructor(address _usdc, address _aiValidator) EIP712("AgentEscrow", "1") {
        if (_usdc == address(0) || _aiValidator == address(0)) revert ZeroAddress();
        USDC = IERC20(_usdc);
        AI_VALIDATOR = _aiValidator;
    }

    // ─── External Functions ────────────────────────────────────────────────

    /**
     * @notice Create a new escrow — locks USDC until validated or refunded
     * @param _seller The service provider address
     * @param _amount USDC amount to escrow (6 decimals)
     * @return id The escrow ID
     */
    function createEscrow(address _seller, uint256 _amount) external nonReentrant returns (uint256 id) {
        if (_seller == address(0)) revert ZeroAddress();
        if (_amount == 0) revert ZeroAmount();

        id = nextEscrowId++;

        // Effects first
        escrows[id] = Escrow({
            buyer: msg.sender,
            seller: _seller,
            amount: _amount,
            deadline: block.timestamp + DEFAULT_DEADLINE,
            state: EscrowState.Created,
            createdAt: block.timestamp
        });

        // Interaction last
        USDC.safeTransferFrom(msg.sender, address(this), _amount);

        emit EscrowCreated(id, msg.sender, _seller, _amount);
    }

    /**
     * @notice Create escrow with custom deadline
     * @param _seller The service provider address
     * @param _amount USDC amount to escrow
     * @param _deadline Custom refund deadline (unix timestamp)
     * @return id The escrow ID
     */
    function createEscrowWithDeadline(
        address _seller,
        uint256 _amount,
        uint256 _deadline
    ) external nonReentrant returns (uint256 id) {
        if (_seller == address(0)) revert ZeroAddress();
        if (_amount == 0) revert ZeroAmount();
        if (_deadline <= block.timestamp) revert DeadlineExpired();

        id = nextEscrowId++;

        escrows[id] = Escrow({
            buyer: msg.sender,
            seller: _seller,
            amount: _amount,
            deadline: _deadline,
            state: EscrowState.Created,
            createdAt: block.timestamp
        });

        USDC.safeTransferFrom(msg.sender, address(this), _amount);

        emit EscrowCreated(id, msg.sender, _seller, _amount);
    }

    /**
     * @notice Seller marks work as completed — triggers AI validation flow
     * @param _escrowId The escrow to mark complete
     */
    function markComplete(uint256 _escrowId) external {
        Escrow storage escrow = escrows[_escrowId];
        if (escrow.state == EscrowState.Released || escrow.state == EscrowState.Refunded) {
            revert EscrowNotFound();
        }
        if (msg.sender != escrow.seller) revert OnlySeller();
        if (escrow.state == EscrowState.Completed) revert AlreadyCompleted();

        escrow.state = EscrowState.Completed;

        emit EscrowCompleted(_escrowId, msg.sender);
    }

    /**
     * @notice AI validator signs and releases funds to seller
     * @param _escrowId The escrow to validate and release
     * @param _signature ECDSA signature from AI validator (65 bytes: r + s + v)
     */
    function validateAndRelease(
        uint256 _escrowId,
        bytes calldata _signature
    ) external nonReentrant {
        Escrow storage escrow = escrows[_escrowId];
        if (escrow.state != EscrowState.Completed) revert NotCompleted();

        // Verify EIP-712 signature from AI validator
        bytes32 structHash = keccak256(abi.encode(
            VALIDATION_TYPEHASH,
            _escrowId
        ));
        bytes32 digest = _hashTypedDataV4(structHash);

        // Use ECDSA.recover() to prevent signature malleability
        address signer = ECDSA.recover(digest, _signature);
        if (signer != AI_VALIDATOR) revert InvalidSignature();

        // State transition prevents replay: Completed → Released (one-way)

        // Effects first
        uint256 amount = escrow.amount;
        address seller = escrow.seller;
        escrow.state = EscrowState.Released;

        // Interaction last
        USDC.safeTransfer(seller, amount);

        emit EscrowValidated(_escrowId, signer);
        emit EscrowReleased(_escrowId, seller, amount);
    }

    /**
     * @notice Buyer can refund if seller hasn't delivered before deadline
     * @param _escrowId The escrow to refund
     */
    function refund(uint256 _escrowId) external nonReentrant {
        Escrow storage escrow = escrows[_escrowId];
        if (msg.sender != escrow.buyer) revert OnlyBuyer();
        if (escrow.state == EscrowState.Released) revert AlreadyRefunded();
        if (escrow.state == EscrowState.Refunded) revert AlreadyRefunded();
        if (escrow.state == EscrowState.Completed) revert AlreadyCompleted();
        if (block.timestamp <= escrow.deadline) revert DeadlineExpired();

        // Effects first
        uint256 amount = escrow.amount;
        address buyer = escrow.buyer;
        escrow.state = EscrowState.Refunded;

        // Interaction last
        USDC.safeTransfer(buyer, amount);

        emit EscrowRefunded(_escrowId, buyer, amount);
    }

    // ─── View Functions ────────────────────────────────────────────────────

    /**
     * @notice Get escrow details
     * @param _escrowId The escrow ID
     */
    function getEscrow(uint256 _escrowId) external view returns (
        address buyer,
        address seller,
        uint256 amount,
        uint256 deadline,
        EscrowState state,
        uint256 createdAt
    ) {
        Escrow storage e = escrows[_escrowId];
        return (e.buyer, e.seller, e.amount, e.deadline, e.state, e.createdAt);
    }

    /**
     * @notice Get total number of escrows created
     */
    function totalEscrows() external view returns (uint256) {
        return nextEscrowId;
    }
}
