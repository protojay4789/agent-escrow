# AgentEscrow — Technical Architecture

> Solana Frontier Hackathon | May 11, 2026
> Sponsor Integrations: Phantom, Swig, Metaplex, World

---

## 1. System Overview

AgentEscrow is a trustless marketplace for AI agent services on Solana. Buyers fund escrow, agents deliver work, and the protocol settles automatically based on oracle-verified completion.

**Core Loop:**
```
Buyer → Phantom wallet → Fund escrow (SOL/USDC)
  → Agent accepts job → Performs work
    → Oracle verifies completion → Funds released
      → Metaplex mints reputation NFT → World signs identity
```

---

## 2. Sponsor Integration Architecture

### 2.1 Phantom — Wallet & UX Layer

**Role:** Primary wallet for agent buyers and sellers.

**Integration Points:**
- `@phantom/solana-wallet-standard` — Wallet adapter for dApp connection
- Deep links for mobile: `https://phantom.app/ul/browse/{url}`
- Transaction signing UX: Escrow fund, release, refund flows
- Embedded iframe for non-custodial buyer onboarding

**Technical:**
```typescript
// Wallet connection
import { useWallet } from '@phantom/solana-wallet-standard';

// Escrow funding transaction
const escrowIx = createEscrowFundInstruction({
  buyer: wallet.publicKey,
  agent: agentWallet,
  amount: lamports,
  jobHash: jobHash,
  vault: escrowPDA,
});
```

**Judge Points:** Embedded UX means buyers never leave the app. Frictionless onboarding.

---

### 2.2 Swig — Payment Routing & Multi-Token Support

**Role:** Smart payment routing across tokens. Agents get paid in their preferred token.

**Integration Points:**
- Swig Wallet SDK for programmable payment splits
- Multi-token escrow: Buyer pays USDC, agent receives SOL (or vice versa)
- Fee routing: Protocol fee → treasury, reputation bonus → agent
- Conditional releases: Milestone-based funding

**Technical:**
```typescript
// Swig payment route
const paymentRoute = await swig.createRoute({
  input: { mint: USDC_MINT, amount: buyerAmount },
  output: { mint: SOL_MINT, recipient: agentWallet },
  slippageBps: 50,
  platformFeeBps: 100, // 1% protocol fee
});
```

**Judge Points:** Multi-token support is real differentiation. Agents shouldn't care what buyers pay with.

---

### 2.3 Metaplex — Soulbound Reputation NFTs

**Role:** Non-transferable reputation tokens for agents. Score based on completion rate, buyer ratings, dispute history.

**Integration Points:**
- Metaplex Core ( Candy Machine v3 ) for minting
- Soulbound tokens (SBTs): `Transfer` instruction disabled
- On-chain metadata: Agent name, completion count, avg rating, specialization
- Compressed NFTs for gas efficiency at scale

**Reputation Schema:**
```typescript
interface AgentReputation {
  address: PublicKey;      // Agent wallet
  name: string;            // Display name
  specialization: string;  // e.g., "code-review", "research", "content"
  jobsCompleted: number;
  avgRating: number;       // 1-5 scale
  disputesLost: number;    // Negative signal
  tier: 'bronze' | 'silver' | 'gold' | 'platinum';
  mintedAt: number;
  lastUpdated: number;
}
```

**Tier Thresholds:**
| Tier | Jobs | Rating | Disputes Lost |
|------|------|--------|---------------|
| Bronze | 0+ | 3.0+ | <5 |
| Silver | 10+ | 4.0+ | <3 |
| Gold | 50+ | 4.5+ | <2 |
| Platinum | 100+ | 4.8+ | 0 |

**Judge Points:** Soulbound reputation is the trust primitive. Agents build portable reputation on Solana.

---

### 2.4 World — Agent Identity Verification

**Role:** Verify that agents are real entities with accountable identities. Prevents Sybil attacks.

**Integration Points:**
- World ID for unique human verification (via World App)
- Orb/phone verification flow for agent operators
- On-chain proof: `WorldId.verify(operator, worldIdHash)` → stores hash in escrow contract
- Optional: Verified badge on agent profile

**Technical:**
```typescript
// World ID verification
const worldProof = await worldClient.getProof({
  action: 'agent-verify',
  signal: agentWallet.toString(),
});

// Store verification on-chain
const verifyIx = createWorldVerifyInstruction({
  agent: agentWallet,
  worldIdHash: worldProof.hashNullifier,
  proof: worldProof.proof,
});
```

**Trust Flow:**
1. Agent operator verifies with World (one-time)
2. Verification linked to agent wallet
3. Buyers see "World Verified" badge
4. Unverified agents can still operate but show lower trust score

**Judge Points:** World verification prevents Sybil and gives buyers confidence. Real identity = real accountability.

---

## 3. Smart Contract Architecture

### 3.1 Program Structure

```
programs/
├── agent_escrow/
│   ├── src/
│   │   ├── lib.rs              // Program entry
│   │   ├── state.rs            // Account structures
│   │   ├── instructions/
│   │   │   ├── fund_escrow.rs  // Buyer funds escrow
│   │   │   ├── accept_job.rs   // Agent accepts work
│   │   │   ├── submit_work.rs  // Agent submits proof
│   │   │   ├── verify_work.rs  // Oracle/DSP verifies
│   │   │   ├── release_funds.rs// Funds released
│   │   │   ├── dispute.rs      // Dispute resolution
│   │   │   └── update_reputation.rs
│   │   └── errors.rs
│   └── Cargo.toml
└── oracle/
    ├── src/
    │   ├── verifier.rs         // Work completion verification
    │   └── reputation.rs       // Score calculation
    └── Cargo.toml
```

### 3.2 Account Structures

```rust
#[account]
pub struct EscrowJob {
    pub buyer: Pubkey,           // Buyer wallet
    pub agent: Pubkey,           // Agent wallet (nullable until accepted)
    pub agent_reputation: Pubkey, // Metaplex NFT account
    pub world_verification: Option<Pubkey>, // World ID proof
    pub amount: u64,             // Escrowed amount (lamports)
    pub token_mint: Pubkey,      // USDC or SOL
    pub job_hash: [u8; 32],      // IPFS/Arweave job spec hash
    pub work_hash: [u8; 32],     // Submitted work hash
    pub status: JobStatus,       // enum
    pub created_at: i64,
    pub deadline: i64,
    pub rating: Option<u8>,      // 1-5 post-completion
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum JobStatus {
    Funded,      // Buyer deposited funds
    Accepted,    // Agent claimed job
    InProgress,  // Agent working
    Submitted,   // Work submitted, pending verification
    Verified,    // Oracle confirmed completion
    Released,    // Funds sent to agent
    Disputed,    // Buyer/agent dispute
    Refunded,    // Funds returned to buyer
    Cancelled,   // Pre-acceptance cancellation
}
```

### 3.3 PDA Seeds

```rust
// Escrow vault
let (vault, bump) = Pubkey::find_program_address(
    &[b"escrow", job.key().as_ref()],
    &program_id,
);

// Reputation account
let (rep_account, bump) = Pubkey::find_program_address(
    &[b"reputation", agent.key().as_ref()],
    &program_id,
);
```

---

## 4. Oracle Layer — Work Verification

### 4.1 Verification Types

| Job Type | Verification Method |
|----------|-------------------|
| Code review | Hash comparison + unit test pass |
| Research | Content hash + citation check |
| Content | Plagiarism scan + quality score |
| Data analysis | Output hash + reproducibility check |
| Design | Client approval + asset hash |

### 4.2 Oracle Flow

```
Agent submits work_hash
  → Oracle checks against job requirements
    → If auto-verifiable: release immediately
      → If needs human review: flag for DSP
        → DSP approves/rejects within 24h
          → Funds released or disputed
```

### 4.3 Dispute Resolution

- Disputes go to a decentralized arbitration panel
- 3 random staked jurors (or DSP members)
- Majority vote decides outcome
- Loser forfeits staked amount to winner

---

## 5. Frontend Architecture

### 5.1 Tech Stack

- **Framework:** Next.js 14 (App Router)
- **Styling:** Tailwind CSS + shadcn/ui
- **State:** Zustand (lightweight)
- **Wallet:** @phantom/solana-wallet-standard
- **Contract:** Anchor (TypeScript bindings)
- **Storage:** Arweave (job specs, work submissions)

### 5.2 Pages

```
/                    → Landing page + value prop
/marketplace         → Browse available jobs
/jobs/[id]           → Job details + escrow flow
/agent/[wallet]      → Agent profile + reputation NFT
/dashboard           → Buyer/agent dashboards
/disputes            → Active dispute management
```

### 5.3 Key UX Flows

**Buyer Flow:**
1. Connect Phantom wallet
2. Create job spec (IPFS/Arweave)
3. Fund escrow (USDC/SOL via Swig)
4. Wait for agent acceptance
5. Review submitted work
6. Approve → funds released, reputation minted

**Agent Flow:**
1. Connect Phantom wallet
2. Browse marketplace
3. Accept job (locks agent to escrow)
4. Perform work
5. Submit work hash + proof
6. Get verified → receive payment + reputation boost

---

## 6. Deployment Plan

### Phase 1: Core (Days 1-2)
- [ ] Anchor program: escrow fund/accept/release
- [ ] Basic Phantom wallet integration
- [ ] Minimal UI: create job, fund, accept, release

### Phase 2: Sponsor Integrations (Days 3-4)
- [ ] Swig payment routing
- [ ] Metaplex reputation NFT minting
- [ ] World ID verification flow
- [ ] Agent profile with reputation display

### Phase 3: Polish (Days 5-6)
- [ ] Oracle verification layer
- [ ] Dispute resolution
- [ ] UI polish + mobile responsive
- [ ] Demo video + pitch deck

### Phase 4: Submit (Day 7)
- [ ] Final testing
- [ ] Submit to Solana Frontier
- [ ] Record demo video
- [ ] Update repo README

---

## 7. Competitive Differentiation

| Feature | AgentEscrow | Traditional Freelance | Other Agent Protocols |
|---------|-------------|----------------------|----------------------|
| Trustless escrow | ✅ On-chain | ❌ Platform trust | ⚠️ Varies |
| Multi-token | ✅ Swig | ❌ Single currency | ❌ Single token |
| Portable reputation | ✅ Soulbound NFT | ❌ Platform-locked | ❌ Centralized |
| Agent identity | ✅ World ID | ❌ Email/phone | ❌ Anonymous |
| AI-native | ✅ Built for agents | ❌ Human-centric | ⚠️ Partial |

---

## 8. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Oracle failure | Fallback to DSP manual review |
| Agent doesn't deliver | Time-locked refund after deadline |
| Buyer doesn't rate | Auto-release after 7 days |
| Sybil attack | World ID verification required for Gold+ |
| Smart contract bug | Audit + bug bounty post-hackathon |

---

*This is the technical foundation. We're building the trust layer for the agent economy.*
