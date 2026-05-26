"""
Tests for AutonomousAgent — OOBE × Ace Data Cloud Bounty

TDD: Tests written FIRST, code verified against them.
Covers: discovery, selection, execution, settlement, verification.
"""

import pytest
from agent.autonomous_agent import (
    AutonomousAgent, AgentConfig, AgentState,
    Service, ExecutionResult
)


# ============================================================
# FIXTURES
# ============================================================

@pytest.fixture
def agent():
    """Fresh agent with sample services."""
    config = AgentConfig(
        address="0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68",
        max_price_usdc=50_000_000,  # $50
    )
    a = AutonomousAgent(config=config)

    a.register_service(Service(
        id=0, provider="0xP1", name="Data Parser",
        description="Parses data", price_usdc=5_000_000,
        capabilities=["parsing", "csv"], active=True,
        total_executions=10, total_revenue=50_000_000,
    ))

    a.register_service(Service(
        id=1, provider="0xP2", name="Image Classifier",
        description="Classifies images", price_usdc=10_000_000,
        capabilities=["classification", "image"], active=True,
        total_executions=20, total_revenue=200_000_000,
    ))

    a.register_service(Service(
        id=2, provider="0xP3", name="Inactive Service",
        description="This is inactive", price_usdc=1_000_000,
        capabilities=["test"], active=False,
        total_executions=0, total_revenue=0,
    ))

    return a


# ============================================================
# DISCOVERY
# ============================================================

class TestDiscovery:
    def test_discover_all_active(self, agent):
        services = agent.discover_services()
        assert len(services) == 2  # One inactive

    def test_discover_by_capability(self, agent):
        services = agent.discover_services(required_capabilities=["parsing"])
        assert len(services) == 1
        assert services[0].name == "Data Parser"

    def test_discover_by_multiple_capabilities(self, agent):
        services = agent.discover_services(required_capabilities=["classification", "image"])
        assert len(services) == 1
        assert services[0].name == "Image Classifier"

    def test_discover_no_match(self, agent):
        services = agent.discover_services(required_capabilities=["translation"])
        assert len(services) == 0

    def test_discover_ignores_inactive(self, agent):
        services = agent.discover_services(required_capabilities=["test"])
        assert len(services) == 0


# ============================================================
# SELECTION
# ============================================================

class TestSelection:
    def test_select_cheapest(self, agent):
        services = agent.discover_services()
        selected = agent.select_service(services)
        assert selected.name == "Data Parser"  # $5 < $10

    def test_select_respects_budget(self, agent):
        agent.config.max_price_usdc = 3_000_000  # $3 max
        services = agent.discover_services()
        selected = agent.select_service(services)
        assert selected is None  # Both too expensive

    def test_select_returns_none_on_empty(self, agent):
        selected = agent.select_service([])
        assert selected is None

    def test_select_prefers_track_record(self, agent):
        # Both same price, pick one with more executions
        agent.register_service(Service(
            id=3, provider="0xP4", name="Cheap Parser",
            description="Also parses", price_usdc=5_000_000,
            capabilities=["parsing"], active=True,
            total_executions=100, total_revenue=500_000_000,
        ))
        services = agent.discover_services(required_capabilities=["parsing"])
        selected = agent.select_service(services)
        # Both $5, but Cheap Parser has more executions
        assert selected.name == "Cheap Parser"


# ============================================================
# EXECUTION
# ============================================================

class TestExecution:
    def test_execute_task(self, agent):
        service = agent.services[0]
        result = agent.execute_task(service)
        assert result.success is True
        assert result.service_id == 0
        assert result.paid_amount == 5_000_000
        assert result.result_hash.startswith("Qm")

    def test_execute_records_history(self, agent):
        service = agent.services[0]
        agent.execute_task(service)
        assert len(agent.execution_history) == 1

    def test_execute_with_input(self, agent):
        service = agent.services[0]
        result = agent.execute_task(service, task_input={"file": "data.csv"})
        assert result.output["task"]["file"] == "data.csv"


# ============================================================
# SETTLEMENT
# ============================================================

class TestSettlement:
    def test_settle_payment(self, agent):
        service = agent.services[0]
        result = agent.execute_task(service)
        settlement = agent.settle_payment(result)
        assert settlement["method"] == "escrow"
        assert settlement["amount"] == result.paid_amount
        assert settlement["tx_hash"].startswith("0x")

    def test_settle_with_router(self, agent):
        service = agent.services[0]
        result = agent.execute_task(service)
        settlement = agent.settle_payment(result, payment_method="router")
        assert settlement["method"] == "router"


# ============================================================
# VERIFICATION
# ============================================================

class TestVerification:
    def test_verify_success(self, agent):
        service = agent.services[0]
        result = agent.execute_task(service)
        settlement = agent.settle_payment(result)
        assert agent.verify_execution(result, settlement) is True
        assert agent.state == AgentState.COMPLETED

    def test_verify_mismatch_amount(self, agent):
        service = agent.services[0]
        result = agent.execute_task(service)
        settlement = agent.settle_payment(result)
        settlement["amount"] = 999_999  # Tamper
        assert agent.verify_execution(result, settlement) is False

    def test_verify_mismatch_service(self, agent):
        service = agent.services[0]
        result = agent.execute_task(service)
        settlement = agent.settle_payment(result)
        settlement["service_id"] = 999  # Tamper
        assert agent.verify_execution(result, settlement) is False


# ============================================================
# FULL LOOP
# ============================================================

class TestFullLoop:
    def test_run_autonomous_loop(self, agent):
        result = agent.run(task_requirements=["parsing"])
        assert result.success is True
        assert result.paid_amount == 5_000_000

    def test_run_raises_on_no_services(self, agent):
        with pytest.raises(ValueError, match="No matching services"):
            agent.run(task_requirements=["nonexistent"])

    def test_run_raises_on_over_budget(self, agent):
        agent.config.max_price_usdc = 1_000_000  # $1
        with pytest.raises(ValueError, match="No affordable services"):
            agent.run(task_requirements=["parsing"])

    def test_run_multiple_times(self, agent):
        agent.run(task_requirements=["parsing"])
        agent.run(task_requirements=["classification"])
        stats = agent.get_stats()
        assert stats["total_executions"] == 2


# ============================================================
# STATS
# ============================================================

class TestStats:
    def test_initial_stats(self, agent):
        stats = agent.get_stats()
        assert stats["total_executions"] == 0
        assert stats["services_available"] == 3  # Includes inactive

    def test_stats_after_execution(self, agent):
        agent.run(task_requirements=["parsing"])
        stats = agent.get_stats()
        assert stats["total_executions"] == 1
        assert stats["successful_executions"] == 1
        assert stats["total_spent"] == 5_000_000
