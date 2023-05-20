#!/bin/bash
echo Runner: Bootstraping
BASE=$(pwd)
echo Runner: $(pwd) is our base path. 
echo "LD_PRELOAD=$BASE/target/debug/libhyperwarphooker.so:$BASE/libhyperglue.so $@" 
LD_PRELOAD="$BASE/target/debug/libhyperwarphooker.so:$BASE/libhyperglue.so" $@
echo Runner: Program ended