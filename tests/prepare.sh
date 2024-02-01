#!/bin/sh

set -e

cd "$(dirname "$0")"

glulxe() {
    echo "Downloading Glulxe"
    rm -rf glulxe
    curl -Ls https://github.com/erkyrath/glulxe/archive/refs/heads/master.tar.gz | tar xz
    mv glulxe-master glulxe -f
    # Set various Makefile options
    sed -i 's|GLKINCLUDEDIR = ../cheapglk|GLKINCLUDEDIR = ../../remglk_capi/src/glk|g' glulxe/Makefile
    sed -i 's|GLKLIBDIR = ../cheapglk|GLKLIBDIR = ../../target/debug|g' glulxe/Makefile
    sed -i 's|GLKMAKEFILE = Make.cheapglk|GLKMAKEFILE = Make.remglk-rs|g' glulxe/Makefile
    sed -i 's|DOS_MAC|DOS_UNIX|g' glulxe/Makefile
    echo "Compiling Glulxe"
    make -C glulxe
}

regtest() {
    echo "Downloading regtest.py"
    curl -s https://raw.githubusercontent.com/erkyrath/plotex/master/regtest.py -o regtest.py
}

for task in "$@"
do
    case "$task" in
        glulxe) glulxe ;;
        regtest) regtest ;;
    esac
done