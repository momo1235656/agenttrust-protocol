// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title DIDRegistry
/// @notice On-chain registry for Decentralized Identifiers (DIDs) following W3C DID Core spec.
/// @dev Stores DID records with Ed25519 public keys, document URIs, and nonce-based versioning.
contract DIDRegistry {
    // -------------------------------------------------------------------------
    // Data Structures
    // -------------------------------------------------------------------------

    struct DIDRecord {
        address controller;       // Ethereum address that controls this DID
        bytes   publicKey;        // Raw Ed25519 public key (32 bytes)
        string  documentURI;      // URI to the DID Document (IPFS / HTTPS)
        bytes32 documentHash;     // keccak256 of the DID Document for integrity
        uint256 createdAt;        // Block timestamp of registration
        uint256 updatedAt;        // Block timestamp of last update
        uint256 nonce;            // Monotonically increasing update counter
        bool    active;           // false after deactivation
    }

    // -------------------------------------------------------------------------
    // State
    // -------------------------------------------------------------------------

    /// @dev did string => DIDRecord
    mapping(string => DIDRecord) private _records;

    /// @dev controller address => list of DIDs they control
    mapping(address => string[]) private _controllerDIDs;

    /// @dev Total number of registered DIDs
    uint256 public totalDIDs;

    // -------------------------------------------------------------------------
    // Events
    // -------------------------------------------------------------------------

    event DIDRegistered(string indexed did, address indexed controller, uint256 timestamp);
    event DIDUpdated(string indexed did, address indexed controller, uint256 nonce, uint256 timestamp);
    event DIDDeactivated(string indexed did, address indexed controller, uint256 timestamp);

    // -------------------------------------------------------------------------
    // Modifiers
    // -------------------------------------------------------------------------

    modifier onlyController(string calldata did) {
        require(_records[did].controller != address(0), "DID not found");
        require(_records[did].controller == msg.sender, "Not the DID controller");
        _;
    }

    modifier mustBeActive(string calldata did) {
        require(_records[did].active, "DID is deactivated");
        _;
    }

    // -------------------------------------------------------------------------
    // External Functions
    // -------------------------------------------------------------------------

    /// @notice Register a new DID.
    /// @param did       The DID string (e.g. "did:key:z6Mk...")
    /// @param publicKey Raw Ed25519 public key (must be exactly 32 bytes)
    /// @param documentURI URI pointing to the DID Document
    /// @param documentHash keccak256 hash of the DID Document
    function register(
        string calldata did,
        bytes calldata publicKey,
        string calldata documentURI,
        bytes32 documentHash
    ) external {
        require(_records[did].controller == address(0), "DID already registered");
        require(publicKey.length == 32, "Invalid Ed25519 public key length");
        require(bytes(did).length > 0, "DID cannot be empty");

        _records[did] = DIDRecord({
            controller:   msg.sender,
            publicKey:    publicKey,
            documentURI:  documentURI,
            documentHash: documentHash,
            createdAt:    block.timestamp,
            updatedAt:    block.timestamp,
            nonce:        0,
            active:       true
        });

        _controllerDIDs[msg.sender].push(did);
        totalDIDs++;

        emit DIDRegistered(did, msg.sender, block.timestamp);
    }

    /// @notice Update the document URI and hash of an existing DID.
    /// @param did          The DID to update
    /// @param documentURI  New URI to the DID Document
    /// @param documentHash keccak256 of the new DID Document
    function update(
        string calldata did,
        string calldata documentURI,
        bytes32 documentHash
    ) external onlyController(did) mustBeActive(did) {
        DIDRecord storage record = _records[did];
        record.documentURI  = documentURI;
        record.documentHash = documentHash;
        record.updatedAt    = block.timestamp;
        record.nonce       += 1;

        emit DIDUpdated(did, msg.sender, record.nonce, block.timestamp);
    }

    /// @notice Permanently deactivate a DID. This action is irreversible.
    /// @param did The DID to deactivate
    function deactivate(string calldata did) external onlyController(did) mustBeActive(did) {
        _records[did].active    = false;
        _records[did].updatedAt = block.timestamp;

        emit DIDDeactivated(did, msg.sender, block.timestamp);
    }

    // -------------------------------------------------------------------------
    // View Functions
    // -------------------------------------------------------------------------

    /// @notice Resolve a DID to its full record.
    /// @param did The DID to resolve
    /// @return The DIDRecord struct
    function resolve(string calldata did) external view returns (DIDRecord memory) {
        require(_records[did].controller != address(0), "DID not found");
        return _records[did];
    }

    /// @notice Check whether a DID is currently active.
    /// @param did The DID to check
    /// @return True if the DID is registered and active
    function isActive(string calldata did) external view returns (bool) {
        return _records[did].controller != address(0) && _records[did].active;
    }

    /// @notice Get all DIDs controlled by a given address.
    /// @param controller The controlling Ethereum address
    /// @return Array of DID strings
    function getDIDsByController(address controller) external view returns (string[] memory) {
        return _controllerDIDs[controller];
    }
}
