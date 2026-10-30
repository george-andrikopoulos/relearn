#!/usr/bin/env bash
# A gate whose function is called below its definition.

check_it() {
    return 0
}

never_called() {
    return 0
}

check_it
