# AgentEscrow

**AI-validated escrow for the agentic economy.**

Agents that pay, validate, and settle — autonomously, on-chain, with proof.

[![Tests](https://img.shields.io/badge/tests-49%2F49%20passing-brightgreen)](./)
[![Solidity](https://img.shields.io/badge/solidity-^0.8.20-blue)](https://soliditylang.org/)
[![Foundry](https://img.shields.io/badge/built%20with-Foundry-FF6C37)](https://book.getfoundry.sh/)

---

## The Problem

AI agents are getting better at *doing* work. But no one has built the financial layer for them to *get paid* for that work — trustlessly, verifiably, and automatically.

**AgentEscrow fixes this.**

---

## What It Does

AgentEscrow is a **two-contract system** that enables autonomous agents to pay for services on-chain, with AI-powered validation before funds move.

| Contract | Purpose |
|----------|---------|
| `AgentEscrow.sol` | USDC escrow with EIP-712 AI validation |
| `TECHPaymentRouter.sol` | Dual-payment routing (USDC + $TECH token burn) |

### Key Features

- **x402-native payments** — HTTP-stablecoin payments, no accounts, no friction
- **EIP-712 signatures** — Gas-efficient, replay-protected AI validator approvals
- **Refund logic** — Buyer can reclaim funds if seller doesn't deliver before deadline
- **Reentrancy protection** — `nonReentrant` + checks-effects-interactions throughout
- **Dual-payment mode** — Pay full USDC, or save 25% using $TECH (burn + treasury split)

---

## How It Works (Agent Flow)

```
┌─────────────┐      x402      ┌──────────────┐
│ AI Agent    │ ──────────────→ │ Service API  │
│ (Buyer)     │                 │ (Seller)     │
└─────────────┘                 └──────────────┘
       │                               │
       │ 1. Lock USDC                  │ 2. Do work
       ▼                               ▼
┌──────────────────────────────────────────────┐
│           AgentEscrow Contract               │
│  - Holds USDC payment                        │
│  - Enforces 7-day deadline (configurable)    │
│  - Waits for AI validator signature          │
└──────────────────────────────────────────────┘
       ▲
       │ 3. EIP-712 signature
       │    (validator approves quality)
┌─────────────┐
│ AI Validator│
│ (Off-chain) │
└─────────────┘
       │
       │ 4. Funds released to seller
       ▼
```

### The Lifecycle

1. **Create** — Buyer locks USDC in escrow, names a seller and deadline
2. **Complete** — Seller marks the job done
3. **Validate** — Off-chain AI validator signs an EIP-712 message approving the work
4. **Release** — Anyone can call `validateAndRelease()` with the signature; funds move to seller instantly
5. **Refund** — If seller never delivers, buyer reclaims USDC after deadline

---

## How to Interact

### As a Developer (Integrate the Contracts)

```solidity
// 1. Create escrow — lock 100 USDC for a service
uint256 escrowId = agentEscrow.createEscrow(
    sellerAddress,    // who will do the work
    100_000000        // 100 USDC (6 decimals)
);

// 2. Seller marks complete
agentEscrow.markComplete(escrowId);

// 3. AI validator signs off-chain (EIP-712)
bytes32 digest = keccak256(abi.encode(
    keccak256("Validation(uint256 escrowId)"),
    escrowId
));
(bytes32 r, bytes32 s, uint8 v) = vm.sign(validatorKey, digest);
bytes memory signature = abi.encodePacked(r, s, v);

// 4. Release funds
agentEscrow.validateAndRelease(escrowId, signature);
```

### As a User (Pay with $TECH Discount)

```solidity
// Pay via TECHPaymentRouter — 25% discount, 50/50 burn/treasury split
// 100 USDC worth of work → only 75 USDC + $TECH fees
techRouter.processPayment(100_000000);
```

### Run the Tests

```bash
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Clone and build
git clone https://github.com/ProtoJay4789/agent-escrow.git
cd agent-escrow
forge build

# Run full test suite (49 tests)
forge test -v
```

**Current status:** `49/49 passing` — AgentEscrow (20) + TECHPaymentRouter (29)

---

## Project Structure

```
contracts/
  AgentEscrow.sol          # Core escrow + EIP-712 validation
  TECHPaymentRouter.sol    # Dual-payment routing with burn logic
test/
  AgentEscrow.t.sol        # 20 tests — create, complete, validate, release, refund
  TECHPaymentRouter.t.sol  # 29 tests — payment splits, ownership, edge cases
script/
  Deploy.s.sol             # Deployment script (Foundry)
docs/
  Kite-AI-Architecture.html  # Interactive architecture diagram
  demo-video-script.md     # 3-min pitch script
```

---

## Architecture

### AgentEscrow.sol

| Function | Access | Purpose |
|----------|--------|---------|
| `createEscrow(seller, amount)` | Buyer | Lock USDC, set 7-day deadline |
| `createEscrowWithDeadline(seller, amount, deadline)` | Buyer | Lock USDC, custom deadline |
| `markComplete(escrowId)` | Seller | Signal work is done |
| `validateAndRelease(escrowId, signature)` | Anyone | Verify EIP-712 sig, release USDC |
| `refund(escrowId)` | Buyer | Reclaim funds after deadline if incomplete |
| `getEscrow(id)` | View | Read escrow state |

### TECHPaymentRouter.sol

| Function | Access | Purpose |
|----------|--------|---------|
| `processPayment(amount)` | Public | Route USDC + burn $TECH |
| `calculateTechAmount(amount)` | View | Preview $TECH needed |
| `updateBurnRatio(bps)` | Owner | Adjust burn/treasury split |
| `updateDiscount(bps)` | Owner | Adjust $TECH discount % |

---

## Stack

- **Contracts:** Solidity ^0.8.20, OpenZeppelin (EIP-712, ReentrancyGuard, SafeERC20)
- **Framework:** Foundry
- **Payments:** x402 pattern (Dexter SDK compatible)
- **Chains:** EVM-native (Avalanche, Base, Polygon — same bytecode)

---

## Roadmap

- [x] AgentEscrow core contract (USDC + EIP-712 + refund)
- [x] TECHPaymentRouter dual-payment system
- [x] 49/49 comprehensive test suite
- [x] Architecture diagram + demo script
- [ ] Kite AI testnet deployment (Chain ID 2368)
- [ ] Kite Attestation integration
- [ ] Public demo UI (Vercel)
- [ ] Multi-chain deployment scripts

---

## License

MIT — see repo for full text.

---

*Built by GenTech Labs. Agents working, agents paid, agents verified.*
