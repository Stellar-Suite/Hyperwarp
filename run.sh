#!/bin/bash
echo Runner: Bootstraping
BASE=$(pwd)
echo Runner: $(pwd) is our base path. $1
echo "LD_PRELOAD=$BASE/libhyperpreglue.so:$BASE/target/debug/libhyperwarphooker.so:$BASE/libhyperglue.so $@" 
cd "`dirname "$1"`"
echo Entered $(pwd), running program. 
LD_LIBRARY_PATH="$BASE/target/debug" LD_PRELOAD="$BASE/libhyperpreglue.so:$BASE/target/debug/libhyperwarphooker.so:$BASE/libhyperglue.so" "$@"
echo Runner: Program ended