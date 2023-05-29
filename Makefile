gluer: glue.c
	gcc -o libhyperglue.so -shared -D_GNU_SOURCE -fPIC glue.c -ldl

pregluer: preglue.c
	gcc -o libhyperpreglue.so -shared -D_GNU_SOURCE -fPIC preglue.c -ldl

rust:
	cargo build

all: rust pregluer gluer
.DEFAULT_GOAL := all