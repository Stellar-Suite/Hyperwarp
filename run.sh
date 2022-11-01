#!/bin/bash
echo Bootstraping
BASE=$(pwd)
echo $(pwd) is our base path. 
LD_PRELOAD="$BASE/target/debug/libhyperwarphooker.so $BASE/libhyperglue.so" $@