fee::deploy() {
    eval "mxpy contract deploy $CALL_ARGS \
        --gas-limit=150000000 \
        --metadata-payable \
        --bytecode='$MY_PARENT_DIR/dex/fee/output/fee.wasm' \
        --outfile='deploy-fee.interaction.json'" 1>/dev/null

    FEE_ADDRESS=$(utils::deployed_address "deploy-fee.interaction.json")
    FEE_ADDRESS_DECODE="0x$(mxpy wallet bech32 --decode $FEE_ADDRESS)"
    TRANSACTION_HASH=$(utils::deployed_tx_hash "deploy-fee.interaction.json")

    [ ! -z "$FEE_ADDRESS" ] && mxpy data store --partition $CHAIN_ID --key=fee-address --value=${FEE_ADDRESS} 1>/dev/null
    [ ! -z "$TRANSACTION_HASH" ] && mxpy data store --partition $CHAIN_ID --key=fee-deploy-tx --value=${TRANSACTION_HASH} 1>/dev/null

    echo "Contract address: $FEE_ADDRESS"
    echo "Deploy transaction hash: $EXPLORER/transactions/$TRANSACTION_HASH"
}

fee::upgrade() {
    eval "mxpy contract upgrade $FEE_ADDRESS $CALL_ARGS \
        --gas-limit=500000000 \
        --bytecode='$MY_PARENT_DIR/dex/fee/output/fee_view.wasm'" 1>/dev/null
}

# params:
#   $1 = fee <= 100_000
#   $2 = address to receive fees
# Example: aggregator::register_ashswap_fee 300 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw
fee::register_ashswap_fee() {
    address="0x$(mxpy wallet bech32 --decode $2)"
    eval "mxpy contract call $FEE_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=registerAshswapFee \
        --arguments $1 $address">/dev/null
}

# params:
#   $1 = token in
#   $2 = address to receive fees
# Example: aggregator::register_protocol_fee 300 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw
fee::register_protocol_fee() {
    address="0x$(mxpy wallet bech32 --decode $2)"
    eval "mxpy contract call $FEE_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=registerProtocolFee \
        --arguments $1 $address">/dev/null
}

# params:
#   $1 = fee <= 100_000
#   $2 = address to receive fees
# Example: aggregator::claim 300 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw
fee::claim() {
    address="0x$(mxpy wallet bech32 --decode $1)"
    eval "mxpy contract call $FEE_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=claimProtocolFee \
        --arguments $address">/dev/null
}

# params:
#   $1 = fee <= 100_000
#   $2 = address to receive fees
# Example: aggregator::ashswap_claim 300 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw
fee::ashswap_claim() {
    eval "mxpy contract call $FEE_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=claimAshswapFee">/dev/null
}