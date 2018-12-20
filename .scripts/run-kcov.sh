#!/usr/bin/env bash
binaries=`ls -l ./target/tests/debug/deps/csv_to-* | grep -v dSYM | grep -v "\.d" | awk '{print $NF}'`
echo "binaries: ${binaries}"
if [[ ! -d "./target/cov" ]]; then
  mkdir -p ./target/cov
fi

ls -lt ./target/cov
for test_bin in $binaries
do
    echo "About to test ${test_bin}"
    kcov ./target/cov/ ${test_bin}
    ls -lt ./target/cov
done