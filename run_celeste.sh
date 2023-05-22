#!/bin/bash
echo Runner: Bootstraping
BASE=$(pwd)
echo Runner: $(pwd) is our base path. 
echo "LD_PRELOAD=$BASE/target/debug/libhyperwarphooker.so:$BASE/libhyperglue.so $@" 
cd "`dirname "$0"`"
echo Entered $(pwd), running program. 
LD_PRELOAD="$BASE/target/debug/libhyperwarphooker.so:$BASE/libhyperglue.so" ./Celeste.bin.x86_64
echo Runner: Program ended