# 1: execute command
# 2: result line
utils::get_result_line() {
    echo "$(eval "$1 | sed -n $2'p'")"
}

# 1: file name
utils::deployed_address() {
    echo $(utils::get_result_line "mxpy data parse --file='$1' --expression='data[\"contractAddress\"]'" 2)
}

# 1: file name
utils::deployed_tx_hash() {
    echo $(utils::get_result_line "mxpy data parse --file='$1' --expression='data[\"transactionOnNetwork\"][\"hash\"]'" 2)
}
