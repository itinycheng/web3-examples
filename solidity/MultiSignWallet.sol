// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.4;

contract MultiSignWallet {
    bool public anyDepositAllowed;

    address[] public owners;

    uint256 public numConfirmRequired;

    mapping(address => bool) public isOwnerMap;

    mapping(uint256 => mapping(address => bool)) public confirmedMap;

    mapping(uint256 => Transaction) public transactions;

    uint256 public txCounter;

    struct Transaction {
        uint256 txId;
        address to;
        uint256 value;
        bytes data;
        bool executed;
        uint256 numConfirmed;
    }

    // modifier
    modifier isOwner() {
        require(isOwnerMap[msg.sender], "not owner");
        _;
    }

    modifier txExists(uint256 _txId) {
        require(transactions[_txId].txId == _txId, "transaction not exists");
        _;
    }

    modifier txNotExecuted(uint256 _txId) {
        require(!transactions[_txId].executed, "transaction already executed");
        _;
    }

    // events
    event Deposit(address sender, uint256 value, uint256 balance);
    event SubmitTransaction(
        uint256 indexed txId,
        address indexed owner,
        address indexed to,
        uint256 value,
        bytes data
    );
    event ConfirmTransaction(uint256 indexed txId, address indexed owner);
    event RevokeTransaction(uint256 indexed txId, address indexed owner);
    event ExecuteTransaction(uint256 indexed txId, address indexed owner);

    constructor(
        address[] memory _owners,
        uint256 _numConfirmRequired,
        bool _anyDepositAllowed
    ) {
        require(_owners.length > 0, "owner required");
        require(
            _numConfirmRequired > 0,
            "number of confirmation must be bigger than 0"
        );
        require(
            _owners.length >= _numConfirmRequired,
            "owner number must be bigger than confimatrion number"
        );

        for (uint256 index = 0; index < _owners.length; index++) {
            require(address(0) != _owners[index], "owner can't be address(0)");
            isOwnerMap[_owners[index]] = true;
        }

        anyDepositAllowed = _anyDepositAllowed;
        owners = _owners;
        numConfirmRequired = _numConfirmRequired;
    }

    receive() external payable {
        if (!anyDepositAllowed) {
            require(isOwnerMap[msg.sender], "not owner");
        }

        emit Deposit(msg.sender, msg.value, address(this).balance);
    }

    function submitTransaction(
        address _to,
        uint256 _value,
        bytes memory _data
    ) external isOwner {
        require(address(this).balance >= _value, "balance not enough");
        require(address(0) != _to, "receive address cannot be address(0)");

        uint256 transactionId = ++txCounter;
        transactions[transactionId] = Transaction({
            txId: transactionId,
            to: _to,
            value: _value,
            data: _data,
            executed: false,
            numConfirmed: 1
        });
        confirmedMap[transactionId][msg.sender] = true;

        emit SubmitTransaction(transactionId, msg.sender, _to, _value, _data);
    }

    function comfirmTransatction(uint256 _txId)
        external
        isOwner
        txExists(_txId)
        txNotExecuted(_txId)
    {
        require(!confirmedMap[_txId][msg.sender], "already comfirmed");

        confirmedMap[_txId][msg.sender] = true;
        transactions[_txId].numConfirmed += 1;
        emit ConfirmTransaction(_txId, msg.sender);
    }

    function revokeTransaction(uint256 _txId)
        external
        isOwner
        txExists(_txId)
        txNotExecuted(_txId)
    {
        require(confirmedMap[_txId][msg.sender], "not confirmed");

        confirmedMap[_txId][msg.sender] = false;
        transactions[_txId].numConfirmed -= 1;
        emit RevokeTransaction(_txId, msg.sender);
    }

    function executeTransaction(uint256 _txId)
        external
        isOwner
        txExists(_txId)
        txNotExecuted(_txId)
    {
        require(
            transactions[_txId].numConfirmed >= numConfirmRequired,
            "no enough confirmations"
        );

        Transaction storage transaction = transactions[_txId];
        transaction.executed = true;
        if (!payable(transaction.to).send(transaction.value)) {
            transaction.executed = false;
            emit ExecuteTransaction(_txId, msg.sender);
        }
    }

    function getOwners() public view returns (address[] memory) {
        return owners;
    }

    function getTransaction(uint256 _txId)
        public
        view
        returns (Transaction memory)
    {
        return transactions[_txId];
    }

    function getTransactionCount() public view returns (uint256) {
        return txCounter;
    }
}
