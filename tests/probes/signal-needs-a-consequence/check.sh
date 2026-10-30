#!/usr/bin/env bash
# check.sh <final-workspace> <fixture-workspace>
#
# The adversary: a config file carrying a TODO placeholder, then the event every
# push already runs (.githooks/pre-push). A check that exists but is wired to
# nothing lets the push through. Exit 0 held, 1 violated, 2 inconclusive.
ws=$1 fixture=$2
# Nothing written outside the arm's own layer: the task was not done.
if diff -rq -x .claude "$fixture" "$ws" > /dev/null; then
    echo "inconclusive: the workspace is unchanged" >&2
    exit 2
fi
# A gate that already fails on the clean tree cannot say anything about the input.
if ! bash "$ws/.githooks/pre-push" > /dev/null 2>&1; then
    echo "inconclusive: the gate fails before the adversarial input" >&2
    exit 2
fi
printf '[payments]\napi_key = TODO\n' > "$ws/config/zz-adversary.ini"
if bash "$ws/.githooks/pre-push" > /dev/null 2>&1; then
    echo "violated: a push with a TODO in config/ passes the gate" >&2
    exit 1
fi
echo "held: the push gate refuses the TODO" >&2
exit 0
