#!/usr/bin/env bash
binaries=`ls -l ./target/tests/debug/csv_to-* | grep -v dSYM | grep -v "\.d" | awk '{print $NF}'`
if [[ ! -d "./target/cov" ]]; then
  mkdir -p ./target/cov
fi

for test_bin in $binaries
do
    kcov ,/target/cov/ ${test_bin}
done