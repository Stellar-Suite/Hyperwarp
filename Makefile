overlay: main.c
	gcc -o liboverlay.so -shared -fPIC main.c -lGL -I/usr/include/freetype2 -lftgl
