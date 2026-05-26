// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title ServiceRegistry
 * @notice On-chain registry where AI agents publish and discover tools/services.
 * @dev Part of OOBE × Ace Data Cloud Bounty submission.
 *
 * Flow:
 * 1. Provider publishes a service (name, description, price, capabilities)
 * 2. Agent discovers services by querying the registry
 * 3. Agent selects a service based on price/capability match
 * 4. Agent pays via AgentEscrow or TECHPaymentRouter
 * 5. Provider delivers result, agent validates, payment settles
 */
contract ServiceRegistry {
    // ── STRUCTS ───────────────────────────────────────────────

    struct Service {
        uint256 id;
        address provider;
        string name;
        string description;
        uint256 priceUSDC;          // Price in USDC (6 decimals)
        string[] capabilities;      // What the service can do
        bool active;
        uint256 createdAt;
        uint256 totalExecutions;
        uint256 totalRevenue;
    }

    struct ExecutionRecord {
        uint256 serviceId;
        address agent;
        uint256 paidAmount;
        bool success;
        uint256 timestamp;
        string resultHash;          // IPFS or on-chain hash of result
    }

    // ── STATE ─────────────────────────────────────────────────

    mapping(uint256 => Service) public services;
    mapping(uint256 => ExecutionRecord[]) public executions;
    uint256 public nextServiceId;
    uint256 public totalServices;
    uint256 public totalExecutions;

    address public owner;

    // ── EVENTS ────────────────────────────────────────────────

    event ServicePublished(
        uint256 indexed serviceId,
        address indexed provider,
        string name,
        uint256 priceUSDC
    );

    event ServiceDeactivated(uint256 indexed serviceId);
    event ServiceReactivated(uint256 indexed serviceId);

    event ExecutionRecorded(
        uint256 indexed serviceId,
        address indexed agent,
        uint256 paidAmount,
        bool success,
        string resultHash
    );

    // ── CONSTRUCTOR ───────────────────────────────────────────

    constructor() {
        owner = msg.sender;
    }

    // ── CORE FUNCTIONS ────────────────────────────────────────

    /**
     * @notice Publish a new service to the registry
     * @param _name Service name (e.g., "Data Parser", "Image Classifier")
     * @param _description What the service does
     * @param _priceUSDC Price in USDC (6 decimals)
     * @param _capabilities List of capability tags
     * @return serviceId The new service ID
     */
    function publishService(
        string calldata _name,
        string calldata _description,
        uint256 _priceUSDC,
        string[] calldata _capabilities
    ) external returns (uint256 serviceId) {
        require(bytes(_name).length > 0, "Name required");

        serviceId = nextServiceId++;

        services[serviceId] = Service({
            id: serviceId,
            provider: msg.sender,
            name: _name,
            description: _description,
            priceUSDC: _priceUSDC,
            capabilities: _capabilities,
            active: true,
            createdAt: block.timestamp,
            totalExecutions: 0,
            totalRevenue: 0
        });

        totalServices++;

        emit ServicePublished(serviceId, msg.sender, _name, _priceUSDC);
    }

    /**
     * @notice Record a successful execution against a service
     * @param _serviceId The service that was executed
     * @param _agent Address of the agent that paid
     * @param _paidAmount Amount paid
     * @param _success Whether execution succeeded
     * @param _resultHash Hash of the result (IPFS CID or similar)
     */
    function recordExecution(
        uint256 _serviceId,
        address _agent,
        uint256 _paidAmount,
        bool _success,
        string calldata _resultHash
    ) external {
        Service storage service = services[_serviceId];
        require(service.active, "Service not active");
        require(_agent != address(0), "Zero address");

        executions[_serviceId].push(ExecutionRecord({
            serviceId: _serviceId,
            agent: _agent,
            paidAmount: _paidAmount,
            success: _success,
            timestamp: block.timestamp,
            resultHash: _resultHash
        }));

        service.totalExecutions++;
        service.totalRevenue += _paidAmount;
        totalExecutions++;

        emit ExecutionRecorded(_serviceId, _agent, _paidAmount, _success, _resultHash);
    }

    /**
     * @notice Deactivate a service (provider only)
     */
    function deactivateService(uint256 _serviceId) external {
        require(services[_serviceId].provider == msg.sender, "Not provider");
        services[_serviceId].active = false;
        emit ServiceDeactivated(_serviceId);
    }

    /**
     * @notice Reactivate a service (provider only)
     */
    function reactivateService(uint256 _serviceId) external {
        require(services[_serviceId].provider == msg.sender, "Not provider");
        services[_serviceId].active = true;
        emit ServiceReactivated(_serviceId);
    }

    // ── VIEW FUNCTIONS ────────────────────────────────────────

    /**
     * @notice Get service details
     */
    function getService(uint256 _serviceId) external view returns (
        uint256 id,
        address provider,
        string memory name,
        string memory description,
        uint256 priceUSDC,
        string[] memory capabilities,
        bool active,
        uint256 createdAt,
        uint256 totalExecutions,
        uint256 totalRevenue
    ) {
        Service storage s = services[_serviceId];
        return (
            s.id,
            s.provider,
            s.name,
            s.description,
            s.priceUSDC,
            s.capabilities,
            s.active,
            s.createdAt,
            s.totalExecutions,
            s.totalRevenue
        );
    }

    /**
     * @notice Get execution count for a service
     */
    function getExecutionCount(uint256 _serviceId) external view returns (uint256) {
        return executions[_serviceId].length;
    }

    /**
     * @notice Get a specific execution record
     */
    function getExecution(uint256 _serviceId, uint256 _index) external view returns (
        address agent,
        uint256 paidAmount,
        bool success,
        uint256 timestamp,
        string memory resultHash
    ) {
        ExecutionRecord storage e = executions[_serviceId][_index];
        return (e.agent, e.paidAmount, e.success, e.timestamp, e.resultHash);
    }

    /**
     * @notice Check if a service has a specific capability
     */
    function hasCapability(uint256 _serviceId, string calldata _capability) external view returns (bool) {
        string[] storage caps = services[_serviceId].capabilities;
        for (uint256 i = 0; i < caps.length; i++) {
            if (keccak256(bytes(caps[i])) == keccak256(bytes(_capability))) {
                return true;
            }
        }
        return false;
    }
}
