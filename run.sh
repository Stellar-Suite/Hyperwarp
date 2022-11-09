#!/bin/bash
echo Runner: Bootstraping
BASE=$(pwd)
echo Runner: $(pwd) is our base path. 
LD_PRELOAD="$BASE/target/debug/libhyperwarphooker.so $BASE/libhyperglue.so" $@