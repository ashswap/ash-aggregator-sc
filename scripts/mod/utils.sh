# 1: execute command
utils::get_last_line() {
    echo "$(eval "$1 | tail -1")"
}

# 1: file name
utils::deployed_address() {
    echo $(utils::get_last_line "mxpy data parse --file='$1' --expression='data[\"contractAddress\"]'")
}

# 1: file name
utils::deployed_tx_hash() {
    echo $(utils::get_last_line "mxpy data parse --file='$1' --expression='data[\"transactionOnNetwork\"][\"hash\"]'")
}
