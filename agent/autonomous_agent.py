"""
Autonomous Agent — OOBE × Ace Data Cloud Bounty

The core agent loop: discover → select → execute → settle → verify.

This agent:
1. Discovers services on-chain via ServiceRegistry
2. Selects a service based on price/capability match
3. Executes the task (simulated for demo, real in production)
4. Settles payment via AgentEscrow or TECHPaymentRouter
5. Records the execution on-chain
6. Produces proof of completion

The agent operates autonomously — no manual hand-holding required.
"""

import time
import json
import hashlib
from dataclasses import dataclass, field
from typing import Optional
from enum import Enum


class AgentState(Enum):
    IDLE = "idle"
    DISCOVERING = "discovering"
    SELECTING = "selecting"
    EXECUTING = "executing"
    SETTLING = "settling"
    VERIFYING = "verifying"
    COMPLETED = "completed"
    FAILED = "failed"


@dataclass
class Service:
    """On-chain service representation."""
    id: int
    provider: str
    name: str
    description: str
    price_usdc: int  # in USDC units (6 decimals)
    capabilities: list[str]
    active: bool
    total_executions: int
    total_revenue: int


@dataclass
class ExecutionResult:
    """Result of a service execution."""
    service_id: int
    agent_address: str
    paid_amount: int
    success: bool
    result_hash: str
    timestamp: float
    execution_time_ms: float
    output: dict = field(default_factory=dict)


@dataclass
class AgentConfig:
    """Agent configuration."""
    address: str
    max_price_usdc: int = 100e6  # Max $100 USDC per execution
    preferred_capabilities: list[str] = field(default_factory=list)
    private_key: str = ""  # For signing transactions
    rpc_url: str = ""  # Solana RPC or EVM RPC


class AutonomousAgent:
    """
    Autonomous agent that discovers, executes, and settles services on-chain.

    Usage:
        agent = AutonomousAgent(config=AgentConfig(address="0x..."))

        # Register available services (for demo)
        agent.register_service(Service(...))

        # Run the autonomous loop
        result = agent.run(task_requirements=["parsing", "classification"])

        # Check result
        if result.success:
            print(f"Executed and paid {result.paid_amount}")
    """

    def __init__(self, config: AgentConfig = None):
        self.config = config or AgentConfig(address="0x0000000000000000000000000000000000000001")
        self.state = AgentState.IDLE
        self.services: dict[int, Service] = {}
        self.execution_history: list[ExecutionResult] = []
        self._execution_counter = 0

    def register_service(self, service: Service):
        """Register a service for discovery (simulates on-chain registry)."""
        self.services[service.id] = service

    def discover_services(self, required_capabilities: list[str] = None) -> list[Service]:
        """
        Discover active services that match capabilities.
        In production, this queries the on-chain ServiceRegistry.
        """
        self.state = AgentState.DISCOVERING
        available = [s for s in self.services.values() if s.active]

        if required_capabilities:
            matched = []
            for service in available:
                service_caps = set(c.lower() for c in service.capabilities)
                required = set(c.lower() for c in required_capabilities)
                if required.issubset(service_caps):
                    matched.append(service)
            available = matched

        return available

    def select_service(self, services: list[Service], task_requirements: dict = None) -> Optional[Service]:
        """
        Select the best service based on price, capabilities, and track record.
        Simple greedy algorithm: cheapest service that meets requirements.
        """
        self.state = AgentState.SELECTING

        if not services:
            return None

        # Filter by max price
        affordable = [s for s in services if s.price_usdc <= self.config.max_price_usdc]

        if not affordable:
            return None

        # Sort by: price (ascending), then by track record (descending)
        affordable.sort(key=lambda s: (s.price_usdc, -s.total_executions))

        return affordable[0]

    def execute_task(self, service: Service, task_input: dict = None) -> ExecutionResult:
        """
        Execute a task using the selected service.
        In production, this calls the service's API/endpoint.
        For demo, we simulate the execution.
        """
        self.state = AgentState.EXECUTING
        start_time = time.time()

        # Simulate execution (in production, this calls the real service)
        output = self._simulate_execution(service, task_input)

        execution_time_ms = (time.time() - start_time) * 1000

        # Generate result hash
        result_data = json.dumps(output, sort_keys=True)
        result_hash = hashlib.sha256(result_data.encode()).hexdigest()[:16]

        result = ExecutionResult(
            service_id=service.id,
            agent_address=self.config.address,
            paid_amount=service.price_usdc,
            success=True,
            result_hash=f"Qm{result_hash}",
            timestamp=time.time(),
            execution_time_ms=execution_time_ms,
            output=output,
        )

        self.execution_history.append(result)
        self._execution_counter += 1

        return result

    def settle_payment(self, result: ExecutionResult, payment_method: str = "escrow") -> dict:
        """
        Settle payment for the executed service.
        In production, this calls AgentEscrow or TECHPaymentRouter.
        """
        self.state = AgentState.SETTLING

        settlement = {
            "method": payment_method,
            "agent": result.agent_address,
            "service_id": result.service_id,
            "amount": result.paid_amount,
            "success": result.success,
            "tx_hash": f"0x{hashlib.sha256(str(time.time()).encode()).hexdigest()[:32]}",
            "timestamp": time.time(),
        }

        return settlement

    def verify_execution(self, result: ExecutionResult, settlement: dict) -> bool:
        """Verify that execution and settlement are consistent."""
        self.state = AgentState.VERIFYING

        # Check settlement matches execution
        if settlement["service_id"] != result.service_id:
            return False
        if settlement["amount"] != result.paid_amount:
            return False
        if not settlement["success"]:
            return False

        self.state = AgentState.COMPLETED
        return True

    def run(self, task_requirements: list[str] = None, task_input: dict = None) -> ExecutionResult:
        """
        Run the full autonomous loop: discover → select → execute → settle → verify.
        This is the main entry point.
        """
        # 1. Discover
        services = self.discover_services(task_requirements)
        if not services:
            self.state = AgentState.FAILED
            raise ValueError("No matching services found")

        # 2. Select
        selected = self.select_service(services)
        if not selected:
            self.state = AgentState.FAILED
            raise ValueError("No affordable services found")

        # 3. Execute
        result = self.execute_task(selected, task_input)

        # 4. Settle
        settlement = self.settle_payment(result)

        # 5. Verify
        verified = self.verify_execution(result, settlement)
        if not verified:
            self.state = AgentState.FAILED
            raise ValueError("Verification failed")

        return result

    def get_stats(self) -> dict:
        """Get agent statistics."""
        return {
            "address": self.config.address,
            "state": self.state.value,
            "total_executions": len(self.execution_history),
            "successful_executions": sum(1 for r in self.execution_history if r.success),
            "total_spent": sum(r.paid_amount for r in self.execution_history),
            "services_available": len(self.services),
        }

    def _simulate_execution(self, service: Service, task_input: dict = None) -> dict:
        """Simulate task execution for demo purposes."""
        return {
            "service": service.name,
            "task": task_input or {"type": "demo", "data": "sample input"},
            "result": f"Processed by {service.name}",
            "capabilities_used": service.capabilities[:2],
            "demo": True,
        }


# ── DEMO SETUP ────────────────────────────────────────────────

def create_demo_agent() -> AutonomousAgent:
    """Create a demo agent with sample services registered."""
    config = AgentConfig(
        address="0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68",
        max_price_usdc=50e6,  # Max $50 per execution
    )

    agent = AutonomousAgent(config=config)

    # Register sample services
    agent.register_service(Service(
        id=0,
        provider="0xProvider1",
        name="Data Parser",
        description="Parses CSV and JSON data into structured formats",
        price_usdc=5e6,  # $5
        capabilities=["parsing", "csv", "json"],
        active=True,
        total_executions=42,
        total_revenue=210e6,
    ))

    agent.register_service(Service(
        id=1,
        provider="0xProvider2",
        name="Image Classifier",
        description="Classifies images using ML models",
        price_usdc=10e6,  # $10
        capabilities=["classification", "image", "ml"],
        active=True,
        total_executions=128,
        total_revenue=1280e6,
    ))

    agent.register_service(Service(
        id=2,
        provider="0xProvider3",
        name="Text Summarizer",
        description="Summarizes long text into key points",
        price_usdc=3e6,  # $3
        capabilities=["summarization", "text", "nlp"],
        active=True,
        total_executions=256,
        total_revenue=768e6,
    ))

    return agent


if __name__ == "__main__":
    # Demo: autonomous agent discovers and executes a service
    agent = create_demo_agent()

    print("=== Autonomous Agent Demo ===\n")
    print(f"Agent: {agent.config.address}")
    print(f"Max budget: ${agent.config.max_price_usdc / 1e6:.0f} USDC\n")

    # Run autonomous loop
    print("Running autonomous loop...")
    result = agent.run(task_requirements=["parsing"])

    print(f"\n--- Execution Result ---")
    print(f"Service: {result.output['service']}")
    print(f"Paid: ${result.paid_amount / 1e6:.0f} USDC")
    print(f"Result hash: {result.result_hash}")
    print(f"Success: {result.success}")

    # Get stats
    stats = agent.get_stats()
    print(f"\n--- Agent Stats ---")
    print(json.dumps(stats, indent=2))
