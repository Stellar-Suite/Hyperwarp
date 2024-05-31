#!/bin/bash
echo Streamer: Starting
BASE=$(pwd)
MODE=debug
HW_DEBUG=1 DEBUG_HW=1 LD_LIBRARY_PATH="$BASE/target/$MODE" LD_PRELOAD="$BASE/libhyperpreglue.so:$BASE/target/$MODE/libhyperwarphooker.so:$BASE/libhyperglue.so" target/rshim
echo Streamer: Exited