# AshSwap Aggregator Smart Contract

This repository contains the Aggregator smart contract in AshSwap.

## Overview

### Aggregator contract
- Aggregator smart contracts act as an intermediary router to exchange tokens between multiple protocols.
- Aggregate function: performs a series of swaps with multiple pools.

<pre>
    fn aggregate(
        steps: ManagedVec(AggregatorStep), -- series of swaps execute sequentially
        limits: ManagedVec(TokenAmount), -- minimum amount out per token
    ) -> ManagedVec<EsdtTokenPayment>; -- results of series of swaps
</pre>
