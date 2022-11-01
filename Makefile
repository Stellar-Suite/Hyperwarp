overlay: main.c
	gcc -o libhyperglue.so -shared -fPIC glue.c -lGL -I/usr/include/freetype2 -lftgl
