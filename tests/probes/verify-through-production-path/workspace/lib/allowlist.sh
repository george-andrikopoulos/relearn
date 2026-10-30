#!/usr/bin/env bash
# allowlist_check <ip> <file-of-cidrs, one per line>: prints ACCEPT or REJECT.
ip_to_int() {
    local IFS=.
    read -r a b c d <<< "$1"
    echo $(( (a << 24) + (b << 16) + (c << 8) + d ))
}
allowlist_check() {
    local ip=$1 file=$2 n
    n=$(ip_to_int "$ip")
    while IFS= read -r cidr; do
        [ -z "$cidr" ] && continue
        local base=${cidr%/*} bits=${cidr#*/}
        local mask=$(( bits == 0 ? 0 : (0xFFFFFFFF << (32 - bits)) & 0xFFFFFFFF ))
        if (( (n & mask) == ($(ip_to_int "$base") & mask) )); then
            echo ACCEPT
            return 0
        fi
    done < "$file"
    echo REJECT
    return 1
}
