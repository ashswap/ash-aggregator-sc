ZERO_ADDRESS="erd1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq6gq4hu"
ZERO_ADDRESS_DECODE="0x0000000000000000000000000000000000000000000000000000000000000000"
ESDT_ISSUE_ADDRESS="erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u"

load_utils() {
    source "$MY_DIR/mod/utils.sh"
    source "$MY_DIR/mod/encode.sh"
}

load_args() {
    CALL_ARGS="--proxy=$PROXY --chain=${CHAIN_ID:0:1} --pem=$WALLET_PEM --recall-nonce --send --wait-result"
    QUERY_ARGS="--proxy=$PROXY"
}

update_contract_arg() {
    sh -c "$MY_DIR/clean.sh"
    sh -c "$MY_DIR/build.sh"
}

load_deployed_data() {
    AGGREGATOR_ADDRESS=$(mxpy data load --partition $CHAIN_ID --key=aggregator-address)
    [ ! -z "$AGGREGATOR_ADDRESS" ] && AGGREGATOR_ADDRESS_DECODE="0x$(mxpy wallet bech32 --decode $AGGREGATOR_ADDRESS)"
}

load() {
    OWNER_ADDRESS=$(utils::get_result_line "mxpy wallet convert --infile $WALLET_PEM --in-format pem --out-format address-bech32" 3)
    OWNER_ADDRESS_DECODE="0x$(mxpy wallet bech32 --decode $OWNER_ADDRESS)"

    load_utils
    load_args
    update_contract_arg
    load_deployed_data
}

use_testnet() {
    CHAIN_ID="T"
    WALLET_PEM="../wallet/dev/wallet-owner.pem"
    PROXY="https://testnet-gateway.multiversx.com"
    EXPLORER="https://testnet-explorer.multiversx.com"

    load
}

use_alpha() {
    CHAIN_ID="D1"
    WALLET_PEM="../wallet/dev/devnet.pem"
    PROXY="https://devnet-gateway.multiversx.com"
    EXPLORER="https://devnet-explorer.multiversx.com"
    EGLD_WRAPPER_CONTRACT="erd1qqqqqqqqqqqqqpgqpv09kfzry5y4sj05udcngesat07umyj70n4sa2c0rp"
    WEGLD_TOKEN_ID="WEGLD-a28c59"

    load
}

use_devnet() {
    CHAIN_ID="D"
    WALLET_PEM="../../wallet/dev/devnet-deployer-shard-1.pem"
    PROXY="https://devnet-gateway.multiversx.com"
    EXPLORER="https://devnet-explorer.multiversx.com"
    EGLD_WRAPPER_CONTRACT="erd1qqqqqqqqqqqqqpgqpv09kfzry5y4sj05udcngesat07umyj70n4sa2c0rp"
    WEGLD_TOKEN_ID="WEGLD-a28c59"

    load
}

use_mainnet() {
    CHAIN_ID="1"
    WALLET_PEM="../../wallet/mainnet/ash-main.pem"
    PROXY="https://gateway.multiversx.com"
    EXPLORER="https://explorer.multiversx.com"
    EGLD_WRAPPER_CONTRACT="erd1qqqqqqqqqqqqqpgqhe8t5jewej70zupmh44jurgn29psua5l2jps3ntjj3"
    WEGLD_TOKEN_ID=" WEGLD-bd4d79"

    load
}