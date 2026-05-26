# Agent Settlement Demo

**OOBE × Ace Data Cloud Bounty Submission**

An autonomous agent that discovers tools on-chain, executes real workflows, and settles payment through escrow — no manual hand-holding required.

## The Problem

AI agents need to discover, use, and pay for services autonomously. Today, this requires manual setup, hardcoded endpoints, and human-mediated payments. There's no standard way for an agent to:

1. Find available services on-chain
2. Evaluate price vs. capability
3. Execute a task
4. Settle payment automatically
5. Prove the work was done

## Our Solution

A complete autonomous agent settlement loop:

```
Discover → Select → Execute → Settle → Verify
```

### How It Works

1. **ServiceRegistry** — Providers publish services on-chain (name, price, capabilities, description)
2. **AutonomousAgent** — Discovers services, selects the best match, executes the task
3. **AgentEscrow** — AI-validated escrow with EIP-712 signatures for secure payment
4. **TECHPaymentRouter** — Dual-payment router with burn/treasury split

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Autonomous Agent                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │ Discover  │→│  Select   │→│ Execute  │→│ Settle │ │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘ │
└─────────────────────────────────────────────────────────┘
        ↓               ↓              ↓            ↓
┌──────────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐
│ ServiceReg.  │ │ Agent    │ │ Provider │ │ Agent      │
│ (on-chain)   │ │ Escrow   │ │ API      │ │ Escrow     │
└──────────────┘ └──────────┘ └──────────┘ └────────────┘
```

## Tech Stack

- **Smart Contracts:** Solidity (Foundry)
- **Agent Loop:** Python
- **Agent Identity:** OOBE Protocol (Solana/Anchor)
- **Payments:** AgentEscrow (USDC) + TECHPaymentRouter ($TECH)
- **Demo:** Vanilla HTML/JS

## Test Results

```
Solidity: 63/63 tests passing
  - AgentEscrow: 20 tests
  - ServiceRegistry: 14 tests
  - TECHPaymentRouter: 29 tests

Python: 23/23 tests passing
  - AutonomousAgent: 23 tests

Total: 86/86 tests passing
```

## Why This Matters

This is the infrastructure for the agent economy:

- **Tool Discovery** — Agents find services without hardcoded endpoints
- **Autonomous Payment** — Agents pay for services without human intervention
- **Trust & Verification** — AI-validated escrow ensures quality
- **Multi-Chain Settlement** — Works across EVM chains and Solana

## Demo

See `demo/index.html` for an interactive demonstration of the autonomous agent loop.

## Running

### Smart Contracts

```bash
cd agent-escrow
forge test
```

### Agent

```bash
cd agent-escrow
python -m pytest tests/test_autonomous_agent.py
```

## Links

- [OOBE Protocol](https://github.com/ProtoJay4789/oobe-protocol)
- [Agent Escrow](https://github.com/ProtoJay4789/agent-escrow)
- [Demo](demo/index.html)

---

Built by GenTech Labs — Agent Economy Infrastructure
