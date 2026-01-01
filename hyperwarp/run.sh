#!/bin/bash
echo Runner: Bootstraping
BASE=$(pwd)
MODE=debug
echo Runner: $(pwd) is our base path. $1
echo "LD_PRELOAD=$BASE/libhyperpreglue.so:$BASE/target/$MODE/libhyperwarphooker.so:$BASE/libhyperglue.so $@" 
cd "`dirname "$1"`"
echo Entered $(pwd), running program. 
HW_DEBUG=1 DEBUG_HW=1 LD_PRELOAD="$BASE/libhyperpreglue.so:$BASE/target/$MODE/libhyperwarphooker.so:$BASE/libhyperglue.so" "$@"
echo Runner: Program ended