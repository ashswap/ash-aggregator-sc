#!/bin/zsh

MY_DIR="$(dirname "$0")"
MY_PARENT_DIR="$(dirname "$MY_DIR")"

## Source all scripts
source "$MY_DIR/mod/load_env.sh"
source "$MY_DIR/mod/aggregator.sh"

## Select env
case $1 in
    "T")
        echo "Use Testnet"
        use_testnet
        ;;
    "D1")
        echo "Use Alpha"
        use_alpha
        ;;
    "D")
        echo "Use Devnet"
        use_devnet
        ;;
    "1")
        echo "Use Mainnet"
        use_mainnet
        ;;
    *)
        echo "Require Elrond chain id (T, D1, D, 1). Ex $0 D" && exit
        ;;
esac