# SUPPORTED TYPE: address, string, bool, u32, u64, biguint
# for ManagedBuffer: address => string, integer => biguint

# $n-odd: data type
# $n-even: data value
encode::nested_encode() {
    data_encode="0x"
    i=0
    for arg in $@
    do
        if [ $((i%2)) -eq 0 ]; then
            data_type=$arg
        else
            data_value=$(encode::encode_by_type $data_type $arg)
            data_encode=$data_encode$data_value
        fi
        i=$i+1
    done
    echo $data_encode
}

# $1: data type
# $2: data value
encode::encode_by_type() {
    case $1 in
        "address")
            echo $(mxpy wallet bech32 --decode $2)
            ;;
        "string")
            value=$2
            if [ ${value:0:2} = "0x" ]; then
                value=${value:2}
            elif [ ${value:0:4} = "erd1" ]; then
                value=$(mxpy wallet bech32 --decode $value)
            else
                value=$(echo -n $value | xxd -p -u | tr -d '\n')
            fi
            echo "$(printf '%08x' $((${#value} / 2)))"$value
            ;;
        "bool")
            echo $(printf '%02x' $2)
            ;;
        "u32")
            echo $(printf '%08x' $2)
            ;;
        "u64")
            echo $(printf '%016x' $2)
            ;;
        "biguint")
            value=$(echo "ibase=10;obase=16;$2" | bc)
            if [ $value = 0 ]; then
                value=""
            elif [ $((${#value} % 2)) -ne 0 ]; then
                value="0$value"
            fi
            echo "$(printf '%08x' $((${#value} / 2)))"$value
            ;;
    esac
}