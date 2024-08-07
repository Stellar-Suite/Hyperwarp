#!/bin/bash
echo Runner: Bootstraping
BASE=$(pwd)
MODE=debug
echo Runner: $(pwd) is our base path. $1
echo "LD_PRELOAD=$BASE/libhyperpreglue.so:$BASE/target/$MODE/libhyperwarphooker.so:$BASE/libhyperglue.so $@" 
cd "`dirname "$1"`"
echo Entered $(pwd), running program. 
# --leak-check=full 
valgrind --trace-children=yes --show-leak-kinds=all --leak-check=full --track-origins=yes --log-file=valgrind-out.txt --track-origins=yes env HW_DEBUG=1 DEBUG_HW=1 LD_LIBRARY_PATH="$BASE/target/$MODE" LD_PRELOAD="$BASE/libhyperpreglue.so:$BASE/target/$MODE/libhyperwarphooker.so:$BASE/libhyperglue.so" "$@"
echo Runner: Program ended