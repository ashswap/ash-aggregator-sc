utils::trim_quote() {
    x_tmp=$(cat)
    x_tmp=${x_tmp#*\"}
    x_tmp=${x_tmp%\"*}
    echo $x_tmp
}