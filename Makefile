gluer: glue.c
	gcc -o libhyperglue.so -shared -D_GNU_SOURCE -fPIC glue.c -ldl

rust:
	cargo build