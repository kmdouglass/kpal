#!/bin/bash
usage="KPAL Test Runner

Usage:
  $(basename "$0") [-h] [-d] <target>... -- run tests associated with a given target(s)

Options:
  -h Show this help text.
  -d The directory containing the test binaries [default: .]
"

OPTIND=1

directory=$(pwd)
while getopts ':h?d:' option; do
  case "$option" in
    h|\?) echo "$usage"
       exit 0
       ;;
    d) directory=$OPTARG
       ;;
  esac
done
shift $((OPTIND - 1))

TEST_BINS=( "$@" )
rc=0
for test in "${TEST_BINS[@]}"; do
    # Filter out files ending in ".d" from the set of filenames.
    cmd=$(find "${directory}" -not -name "*.d" -name "${test}-*")

    echo "***Running test binary: ${cmd}" >&2
    eval "${cmd}"
    status=$?

    if [ "${status}" -ne 0 ] && [ "${rc}" -eq 0 ]; then
	rc=1
    fi
done

if [ "${rc}" -ne 0 ]; then
    echo "***FAILURE" >&2
    exit "${rc}"
fi

echo "***OK" >&2
exit 0
