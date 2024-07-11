gluer: glue.c
	gcc -o libhyperglue.so -O3 -shared -D_GNU_SOURCE -fPIC glue.c -ldl

pregluer: preglue.c
	gcc -o libhyperpreglue.so -O3 -shared -D_GNU_SOURCE -fPIC preglue.c -ldl

rshim: shim.c
	gcc -o target/rshim -O3 -D_GNU_SOURCE shim.c

rust:
	cargo build --features "log_dlsym"

rust_release:
	cargo build -r

all: pregluer gluer rshim rust rust_release
.DEFAULT_GOAL := all