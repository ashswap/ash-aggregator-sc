wrapper::deploy() {
    eval "mxpy contract deploy $CALL_ARGS \
        --gas-limit=150000000 \
        --bytecode='$MY_PARENT_DIR/dex/wrapper_view/output/wrapper_view.wasm' \
        --outfile='deploy-wrapper.interaction.json'" 1>/dev/null

    WRAPPER_ADDRESS=$(utils::deployed_address "deploy-wrapper.interaction.json")
    WRAPPER_ADDRESS_DECODE="0x$(mxpy wallet bech32 --decode $WRAPPER_ADDRESS)"
    TRANSACTION_HASH=$(utils::deployed_tx_hash "deploy-wrapper.interaction.json")

    [ ! -z "$WRAPPER_ADDRESS" ] && mxpy data store --partition $CHAIN_ID --key=wrapper-address --value=${WRAPPER_ADDRESS} 1>/dev/null
    [ ! -z "$TRANSACTION_HASH" ] && mxpy data store --partition $CHAIN_ID --key=wrapper-deploy-tx --value=${TRANSACTION_HASH} 1>/dev/null
}

wrapper::upgrade() {
    eval "mxpy contract upgrade $WRAPPER_ADDRESS $CALL_ARGS \
        --gas-limit=500000000 \
        --bytecode='$MY_PARENT_DIR/dex/wrapper_view/output/wrapper_view.wasm'" 1>/dev/null
}

wrapper::get_ashswapv1() {
    address="0x$(mxpy wallet bech32 --decode $1)"
    eval "mxpy contract query $DELEGATE_ADDRESS $QUERY_ARGS \
        --function=getAshswapV1 \
        --arguments $address"
}

wrapper::get_ashswapv2() {
    address="0x$(mxpy wallet bech32 --decode $1)"
    eval "mxpy contract query $DELEGATE_ADDRESS $QUERY_ARGS \
        --function=getAshswapV2 \
        --arguments $address"
}

wrapper::get_jexexchange() {
    address="0x$(mxpy wallet bech32 --decode $1)"
    eval "mxpy contract query $DELEGATE_ADDRESS $QUERY_ARGS \
        --function=getJexExchange \
        --arguments $address"
}

wrapper::get_onedex() {
    address="0x$(mxpy wallet bech32 --decode $1)"
    token_0="0x$(echo -n $2 | xxd -p -u | tr -d '\n')"
    token_1="0x$(echo -n $3 | xxd -p -u | tr -d '\n')"
    eval "mxpy contract query $DELEGATE_ADDRESS $QUERY_ARGS \
        --function=getOnedex \
        --arguments $address $token_0 $token_1"
}

wrapper::get_xexchange() {
    address="0x$(mxpy wallet bech32 --decode $1)"
    token_0="0x$(echo -n $2 | xxd -p -u | tr -d '\n')"
    token_1="0x$(echo -n $3 | xxd -p -u | tr -d '\n')"
    eval "mxpy contract query $DELEGATE_ADDRESS $QUERY_ARGS \
        --function=getXExchange \
        --arguments $address $token_0 $token_1"
}
