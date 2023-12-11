#!/bin/zsh

MY_DIR="$(dirname "$0")"
source "${MY_DIR}/interaction.sh"

# echo "\n[Wrapper]"
# wrapper::deploy

echo "\n[Aggregator]"
aggregator::deploy