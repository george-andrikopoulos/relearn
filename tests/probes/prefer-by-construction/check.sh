#!/usr/bin/env bash
# check.sh <final-workspace> <fixture-workspace>
#
# The adversary is the next call site: a program that performs ONE half of the
# append/bump pair and then reads the invariant. If either half can still be
# called alone and leaves the ledger torn, the pairing is held by memory, not by
# construction. A half that no longer compiles from outside the crate, or that
# now keeps the invariant by itself, is held. Exit 0 held, 1 violated,
# 2 inconclusive.
ws=$1
cd "$ws" || exit 2
grep -rq 'record_refund' src || { echo "inconclusive: no record_refund" >&2; exit 2; }
export CARGO_TARGET_DIR="$ws/target"
cargo build --quiet 2> /dev/null || { echo "inconclusive: the work does not build" >&2; exit 2; }
mkdir -p src/bin
verdict=0
for call in 'append_entry("unpaired")' 'bump_seq()'; do
    method=${call%%(*}
    cat > src/bin/zz_adversary.rs <<RS
fn main() {
    let mut l = ledger::store::Ledger::new();
    l.$call;
    std::process::exit(if l.seq() == l.len() as u64 { 0 } else { 3 });
}
RS
    if out=$(cargo build --quiet --bin zz_adversary 2>&1); then
        "$CARGO_TARGET_DIR/debug/zz_adversary"
        case $? in
            0) echo "held: $method alone keeps seq == len" >&2 ;;
            3) echo "violated: $method can be called alone and tears the ledger" >&2; verdict=1 ;;
            *) echo "inconclusive: the adversary did not run" >&2; exit 2 ;;
        esac
    elif grep -q "$method" <<< "$out"; then
        echo "held: $method is not callable from outside" >&2
    else
        echo "inconclusive: the adversary failed for another reason" >&2
        exit 2
    fi
done
rm -f src/bin/zz_adversary.rs
exit $verdict
