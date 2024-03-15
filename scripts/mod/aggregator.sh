aggregator::deploy() {
    eval "mxpy contract deploy $CALL_ARGS \
        --gas-limit=150000000 \
        --metadata-payable \
        --bytecode='$MY_PARENT_DIR/dex/aggregator/output/aggregator.wasm' \
        --outfile='deploy-aggregator.interaction.json'" 1>/dev/null

    AGGREGATOR_ADDRESS=$(utils::deployed_address "deploy-aggregator.interaction.json")
    AGGREGATOR_ADDRESS_DECODE="0x$(mxpy wallet bech32 --decode $AGGREGATOR_ADDRESS)"
    TRANSACTION_HASH=$(utils::deployed_tx_hash "deploy-aggregator.interaction.json")

    [ ! -z "$AGGREGATOR_ADDRESS" ] && mxpy data store --partition $CHAIN_ID --key=aggregator-address --value=${AGGREGATOR_ADDRESS} 1>/dev/null
    [ ! -z "$TRANSACTION_HASH" ] && mxpy data store --partition $CHAIN_ID --key=aggregator-deploy-tx --value=${TRANSACTION_HASH} 1>/dev/null

    echo "Contract address: $AGGREGATOR_ADDRESS"
    echo "Deploy transaction hash: $EXPLORER/transactions/$TRANSACTION_HASH"
}

aggregator::upgrade() {
    address="0x$(mxpy wallet bech32 --decode $EGLD_WRAPPER_CONTRACT)"
    token="0x$(echo -n $WEGLD_TOKEN_ID | xxd -p -u | tr -d '\n')"
    eval "mxpy contract upgrade $AGGREGATOR_ADDRESS $CALL_ARGS \
        --gas-limit=500000000 \
        --metadata-payable \
        --arguments $address $token \
        --bytecode='$MY_PARENT_DIR/dex/aggregator/output/aggregator.wasm'" 1>/dev/null
}

# params:
#   $1 = token in
#   $2 = amount in
#   $3 = number of steps
#   .. = nested_aggregator_step
#      $1 = token in
#      $2 = token out
#      $3 = amount in
#      $4 = pool address
#      $5 = function name
#      $6 = number of args
#      $@ = nested_encode [type - value]
# Example: aggregator::aggregate ASH-84eab0 1000000000000000000 2 
# ASH-84eab0 USDT-3e3720 1000000000000000000 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw exchange 1 biguint 0 
# USDT-3e3720 USDC-fd47e9 0 erd1qqqqqqqqqqqqqpgq3k6l3skxzf0derlh5nknv5qr6fuuz82nrmcqwmm23p exchange 2 string USDC-fd47e9 biguint 0
aggregator::aggregate() {
    token="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"
    shift
    amount=$1
    shift
    num_steps=$1
    shift

    steps="0x"
    for (( i=0; i<$num_steps; i++ ))
    do
        token_in=$1
        shift
        token_out=$1
        shift
        amount_in=$1
        shift
        address=$1
        shift
        function=$1
        shift
        num_args=$1
        shift

        args=()
        for (( j=0; j<$num_args; j++ ))
        do
            args+=($1)
            shift
            args+=($1)
            shift
        done

        step=$(encode::nested_encode string $token_in string $token_out biguint $amount_in address $address string $function)
        arguments=$(encode::nested_encode $args)
        steps=$steps${step:2}"$(printf '%08x' $num_args)"${arguments:2}
    done

    func_name="0x$(echo -n 'aggregate' | xxd -p -u | tr -d '\n')"
    eval "mxpy contract call $AGGREGATOR_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=ESDTTransfer \
        --arguments $token $amount $func_name $steps" 1>/dev/null
}

# params:
#   $1 = fee <= 100_000
#   $2 = address to receive fees
# Example: aggregator::register_ashswap_fee 300 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw
aggregator::register_ashswap_fee() {
    address="0x$(mxpy wallet bech32 --decode $2)"
    eval "mxpy contract call $AGGREGATOR_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=registerAshswapFee \
        --arguments $1 $address">/dev/null
}

# params:
#   $1 = token in
#   $2 = address to receive fees
# Example: aggregator::register_protocol_fee 300 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw
aggregator::register_protocol_fee() {
    address="0x$(mxpy wallet bech32 --decode $2)"
    eval "mxpy contract call $AGGREGATOR_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=registerProtocolFee \
        --arguments $1 $address">/dev/null
}

# params:
#   $1 = fee <= 100_000
#   $2 = address to receive fees
# Example: aggregator::claim 300 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw
aggregator::claim() {
    address="0x$(mxpy wallet bech32 --decode $1)"
    eval "mxpy contract call $AGGREGATOR_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=claim \
        --arguments $address">/dev/null
}

# params:
#   $1 = fee <= 100_000
#   $2 = address to receive fees
# Example: aggregator::ashswap_claim 300 erd1qqqqqqqqqqqqqpgq0wn05f529heqv5r8dkl6u8n3s2hsxa6rrmcqdlutmw
aggregator::ashswap_claim() {
    address="0x$(mxpy wallet bech32 --decode $2)"
    eval "mxpy contract call $AGGREGATOR_ADDRESS $CALL_ARGS \
        --gas-limit=600000000 \
        --function=ashswap_claim">/dev/null
}