#!/bin/bash

cd "$(dirname "$0")"

FAILURES=0

run_test() {
    python regtest.py -E RUST_BACKTRACE=1 -i "./glulxe/glulxe" -r -t ${2:-10} $1 || ((FAILURES++))
}

echo 'Glulx tests'
echo ' Glulxercise'
run_test glulxercise.ulx.regtest
echo ' advent'
run_test advent.ulx.regtest
rm adventtest.glksave

echo 'Glk tests'
echo ' datetimetest'
run_test datetimetest.ulx.regtest
echo ' extbinaryfile'
run_test extbinaryfile.ulx.regtest
rm binfile.glkdata
echo ' externalfile'
run_test externalfile.ulx.regtest
rm testfile*
echo ' graphwintest'
# TODO: support refresh
run_test graphwintest.gblorb.regtest
echo ' imagetest'
# TODO: support refresh
run_test imagetest.gblorb.regtest
echo ' inputeventtest'
run_test inputeventtest.ulx.regtest
echo ' inputfeaturetest'
run_test inputfeaturetest.ulx.regtest
echo ' memstreamtest'
run_test memstreamtest.ulx.regtest
echo ' resstreamtest'
run_test resstreamtest.gblorb.regtest
echo ' startsavetest'
run_test startsavetest.gblorb.regtest
echo ' unicasetest'
run_test unicasetest.ulx.regtest
echo ' unicodetest'
run_test unicodetest.ulx.regtest
echo ' windowtest'
run_test windowtest.ulx.regtest

exit $FAILURES