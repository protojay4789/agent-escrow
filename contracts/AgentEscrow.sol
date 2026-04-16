// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";

/**
 * @title AgentEscrow
 * @notice AI-validated escrow for agent-to-service payments.
 * @dev Holds USDC, validates work via AI agent signatures, releases or refunds.
 */
contract AgentEscrow is EIP712 {
    IERC20 public immutable usdc;
    address public immutable aiValidator;

    struct Escrow {
        address buyer;
        address seller;
        uint256 amount;
        bool completed;
        bool validated;
    }

    mapping(uint256 => Escrow) public escrows;
    uint256 public nextEscrowId;

    bytes32 constant VALIDATION_TYPEHASH = keccak256(
        "Validation(uint256 escrowId, address validator, uint256 timestamp)"
    );

    event EscrowCreated(uint256 indexed id, address indexed buyer, address indexed seller, uint256 amount);
    event EscrowValidated(uint256 indexed id, address validator);
    event EscrowReleased(uint256 indexed id);
    event EscrowRefunded(uint256 indexed id);

    constructor(address _usdc, address _aiValidator) EIP712("AgentEscrow", "1") {
        usdc = IERC20(_usdc);
        aiValidator = _aiValidator;
    }

    function createEscrow(address _seller, uint256 _amount) external returns (uint256) {
        usdc.transferFrom(msg.sender, address(this), _amount);
        
        uint256 id = nextEscrowId++;
        escrows[id] = Escrow({
            buyer: msg.sender,
            seller: _seller,
            amount: _amount,
            completed: false,
            validated: false
        });
        
        emit EscrowCreated(id, msg.sender, _seller, _amount);
        return id;
    }

    function markComplete(uint256 _escrowId) external {
        Escrow storage escrow = escrows[_escrowId];
        require(msg.sender == escrow.seller, "Only seller");
        require(!escrow.completed, "Already completed");
        escrow.completed = true;
    }

    function validateAndRelease(
        uint256 _escrowId,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        Escrow storage escrow = escrows[_escrowId];
        require(escrow.completed, "Not completed");
        require(!escrow.validated, "Already validated");

        // Verify AI validator signature (EIP-712)
        bytes32 structHash = keccak256(abi.encode(
            VALIDATION_TYPEHASH,
            _escrowId,
            aiValidator,
            block.timestamp
        ));
        bytes32 digest = _hashTypedDataV4(structHash);
        address signer = ecrecover(digest, v, r, s);
        require(signer == aiValidator, "Invalid validator signature");

        escrow.validated = true;
        usdc.transfer(escrow.seller, escrow.amount);
        
        emit EscrowValidated(_escrowId, signer);
        emit EscrowReleased(_escrowId);
    }

    function refund(uint256 _escrowId) external {
        Escrow storage escrow = escrows[_escrowId];
        require(msg.sender == escrow.buyer, "Only buyer");
        require(!escrow.completed, "Already completed — use validate flow");
        
        usdc.transfer(escrow.buyer, escrow.amount);
        emit EscrowRefunded(_escrowId);
    }
}
