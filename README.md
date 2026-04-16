# AgentEscrow

**AI-validated escrow with x402 payments for the agentic economy.**

Agents that pay, validate, and settle — autonomously, on-chain.

## What Is This?

AgentEscrow combines three primitives:
1. **x402 payments** — HTTP-native stablecoin payments (no accounts, no friction)
2. **AI validation** — Autonomous agents verify work quality before releasing funds
3. **Smart contract escrow** — Trustless fund holding with programmable release conditions

## Architecture

```
┌─────────────┐      x402      ┌──────────────┐
│ AI Agent    │ ──────────────→ │ Service API  │
│ (Buyer)     │                 │ (Seller)     │
└─────────────┘                 └──────────────┘
       │                               │
       │ Payment                       │ Work completion
       ▼                               ▼
┌──────────────────────────────────────────────┐
│           AgentEscrow Contract               │
│  - Holds USDC payment                        │
│  - AI validator validates work               │
│  - Releases funds or refunds                 │
└──────────────────────────────────────────────┘
```

## Stack

- **Smart Contracts:** Solidity (Foundry)
- **Payments:** Dexter x402 SDK v3.0
- **Identity:** ERC-8004 (Agent Registration)
- **Jobs:** ERC-8183 (Agent Jobs)
- **Validation:** EIP-712 signatures
- **Chains:** Avalanche, Base, Polygon (EVM-compatible)

## Project Structure

```
contracts/       Solidity smart contracts
src/             TypeScript application code
  agents/        Agent logic and identity
  payment/       x402 payment integration
test/            Contract and integration tests
docs/            Documentation and diagrams
scripts/         Deployment and utility scripts
```

## Development

```bash
# Install dependencies
npm install

# Compile contracts
forge build

# Run tests
forge test

# Deploy (Avalanche Fuji testnet)
forge script scripts/Deploy.scr --rpc-url fuji --broadcast
```

## Roadmap

- [ ] AgentEscrow core contract (USDC + EIP-712)
- [ ] x402 payment middleware integration
- [ ] Agent identity registration (ERC-8004)
- [ ] End-to-end demo: Agent pays → Escrow → Validate → Release
- [ ] Kite AI deployment (when ready)
- [ ] Multi-chain support

## License

MIT
