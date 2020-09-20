#!/bin/bash

if [ -z "$BASH_SOURCE" ]; then
    echo This script must be source-d.
    exit 1
fi

function main () {
    SCRIPT_DIR=$(cd $(dirname $BASH_SOURCE); pwd)
    if [ -z "$IDF_PATH" ]; then
        echo IDF_PATH environment variable must be set.
        return 1
    fi
    if [ ! -f $IDF_PATH/export.sh ]; then
        echo $IDF_PATH/export.sh does not exist. Run $IDF_PATH/install.sh and set up ESP-IDF at first.
        return 1
    fi

    if [ ! -d $SCRIPT_DIR/env ]; then
        python3 -m venv $SCRIPT_DIR/env
        source $SCRIPT_DIR/env/bin/activate
        pip install wheel
        pip install -r $IDF_PATH/requirements.txt
    else
        source env/bin/activate
    fi

    source $IDF_PATH/export.sh

    export WORKDIR=$HOME/rust-esp32
    export XTENSA_TOOLCHAIN_ROOT=$HOME/.espressif/tools/xtensa-esp32-elf/esp-2019r2-8.2.0/xtensa-esp32-elf
    export XARGO_RUST_SRC=$WORKDIR/rust-xtensa/library
    export LIBCLANG_PATH=$WORKDIR/build/llvm-project/lib
    export PROJECT_BUILD_INCLUDE_PATH=${SCRIPT_DIR}/guruguru_rust/build/config
}

main