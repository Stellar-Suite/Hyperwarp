# TODO
* Make compiling with gl optional. 
* Setup io threads for each transport, they'll parse and validate each message before our busy thread checks it's own special queue. 
* Vulkan support?
* Support the vastly more efficient xcb XPixmap sharing thing see [this usage of obs-vkcapture](https://github.com/nowrep/obs-vkcapture/blob/eb4b07b75d13218877b16adc20ff8fdd28c02f5e/src/glinject.c#L874). Very important for performance.
* also see [this stackoverflow](https://stackoverflow.com/questions/36843456/read-pixel-data-from-default-framebuffer-in-opengl-performance-of-fbo-vs-pbo) (thanks @anirudhb).
* Update laptop to try getting this to work efficiently on Wayland.
* sdl3 now exists, see [migration guide](https://github.com/libsdl-org/SDL/blob/main/docs/README-migration.md)

Some people have diff pixel density:

> SDL_DisplayMode now includes the pixel density which can be greater than 1.0 for display modes that have a higher pixel size than the mode size. You should use SDL_GetWindowSizeInPixels() to get the actual pixel size of the window back buffer.