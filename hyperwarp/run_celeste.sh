#!/bin/bash
echo Runner: Bootstraping to $1
BASE=$(pwd)
echo Runner: $(pwd) is our base path. 
echo "LD_PRELOAD=$BASE/libhyperpreglue.so:$BASE/target/debug/libhyperwarphooker.so:$BASE/libhyperglue.so $@" 
cd "`dirname "$1"`"
echo Entered $(pwd), running program. 
LD_LIBRARY_PATH="$BASE/target/debug" LD_PRELOAD="$BASE/libhyperpreglue.so:$BASE/target/debug/libhyperwarphooker.so:$BASE/libhyperglue.so" ./Celeste.bin.x86_64
echo Runner: Program ended