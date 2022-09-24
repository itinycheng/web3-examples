// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.4 <0.9.0;

contract Auction {
    address payable public immutable beneficiary;

    address public highestBidder;
    uint256 public highestBid;

    mapping(address => uint256) pendingReturns;

    uint256 endTime;

    bool ended;

    event HighestBidLog(address bidder, uint256 amount);
    event AuctionEnded(address winer, uint256 amount);

    error AuctionNotEnd();
    error BidNotHighEnough();
    error AuctionAlreadyEnded();
    error AuctionEndCalled();

    constructor(uint256 duration, address payable _beneficiary) {
        uint256 blockTime = block.timestamp;
        beneficiary = _beneficiary;
        endTime = blockTime + duration;
    }

    function bid() external payable {
        if (block.timestamp > endTime) {
            revert AuctionAlreadyEnded();
        }

        uint256 bidAmount = pendingReturns[msg.sender] + msg.value;
        if (bidAmount <= highestBid) {
            revert BidNotHighEnough();
        }

        pendingReturns[msg.sender] = bidAmount;
        highestBidder = msg.sender;
        highestBid = bidAmount;
        emit HighestBidLog(msg.sender, bidAmount);
    }

    function withdraw() external returns (bool) {
        require(msg.sender != highestBidder, "highest bidder");

        if (block.timestamp <= endTime) {
            revert AuctionNotEnd();
        }

        uint256 bidAmount = pendingReturns[msg.sender];
        if (bidAmount > 0) {
            pendingReturns[msg.sender] = 0;
            if (!payable(msg.sender).send(bidAmount)) {
                pendingReturns[msg.sender] = bidAmount;
                return false;
            }
        }

        return true;
    }

    function auctionEnd() external {
        if (block.timestamp <= endTime) {
            revert AuctionNotEnd();
        }

        if (ended) {
            revert AuctionEndCalled();
        }

        beneficiary.transfer(highestBid);
        ended = true;
        emit AuctionEnded(highestBidder, highestBid);
    }
}
