---
date: 2026-05-05
author: Jordan / DMOB
status: DRAFT — awaiting Jordan approval
platforms: [x, facebook]
project: AgentEscrow — Colosseum Solana Frontier
tags: [hackathon, solana, agents, escrow, x402]
---

# 🐦 X Post (Technical / Crypto Audience)

**Option A — Hook Thread (4 posts):**

**1/4**
AI agents can't pay for things online.

The web was built for humans — accounts, credit cards, OAuth. When an autonomous agent needs to buy data or pay another agent for work, there's no native payment layer.

We're fixing that. 🧵

**2/4**
Meet AgentEscrow — an escrow protocol built natively on Solana.

Agents pay per-request via HTTP 402 ("Payment Required") — the web's missing status code. Funds lock in on-chain escrow. Release happens when work is verified. Auto-refund on timeout.

No accounts. No subscriptions. No human in the loop.

**3/4**
Why Solana?
→ 400ms finality (agents can't wait 12s)
→ $0.00025 per tx (sub-cent agent calls are viable)
→ 37M+ x402 transactions already flowing
→ Sealevel parallel execution = 1000 agents paying simultaneously

This is where the agent economy already lives.

**4/4**
Building for @Colosseum_sol Frontier hackathon. 1,150 lines of Anchor/Rust. 4 programs: Agent Registry, Job Escrow, Dispute Resolver, Reputation.

"Because agents can't shake hands. They need escrow."

🔗 [link to submission]

---

**Option B — Single Post (standalone):**

AI agents can't pay for things online.

We built AgentEscrow — an escrow protocol on Solana where agents pay per-request via HTTP 402, funds lock on-chain, and release happens when work is verified.

1,150 lines of Anchor/Rust. 4 programs. Built for @Colosseum_sol Frontier.

Because agents can't shake hands. They need escrow.

---

# 📘 Facebook Post (Non-Technical / Friends & Family)

**Hey everyone! 👋**

Wanted to share something cool we've been working on.

You know how AI agents (like ChatGPT, but ones that can actually DO things) are becoming a bigger deal? Well, there's a problem — when one AI agent needs to hire another AI agent to do work, there's no way to handle the payment safely.

Imagine hiring someone on Fiverr, but there's no Fiverr. No way to hold the money until the work is done. No protection if someone ghosts you. That's the situation AI agents are in right now.

So we built **AgentEscrow** — think of it as a smart payment system for AI agents. One agent says "hey, do this task" and puts up the money. The other agent does the work. Once the work is verified, the money releases automatically. If something goes wrong, the money goes back to the buyer.

We're competing in a Solana hackathon with $230K+ in prizes. The code is written in Rust (a super安全 language), it's been audited, and we're deploying it this week.

It's wild that we're building infrastructure for a future where AI agents hire each other. But that future is coming fast, and someone's gotta build the trust layer. 🤝

Wish us luck! 🍀

---

# Usage Notes

- **X**: Post Option B standalone first, save thread for a follow-up or reply chain
- **Facebook**: Post as-is, add a screenshot of the architecture diagram or a terminal build if DMOB can produce one
- **Timing**: Post X during peak hours (10am-12pm EST or 2pm-4pm EST). Facebook anytime.
- **Media**: Attach a clean terminal screenshot of `anchor build` succeeding + the architecture diagram if available
