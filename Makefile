gluer: glue.c
	gcc -o libhyperglue.so -O3 -shared -D_GNU_SOURCE -fPIC glue.c -ldl

pregluer: preglue.c
	gcc -o libhyperpreglue.so -O3 -shared -D_GNU_SOURCE -fPIC preglue.c -ldl

rust:
	cargo build

rust_release:
	cargo build -r

all: pregluer gluer rust rust_release
.DEFAULT_GOAL := all