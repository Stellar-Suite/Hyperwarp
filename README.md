# Hyperwarp
Disclaimer: THIS CODE IS EXTREMELY CURSED. NO QUALITY GUARANTEE IS PROVIDED. USE AT YOUR OWN RISK.

Please do not open PRs at the time since a lot of things are constantly changing and there is like 0 stability guarantees.

## notes
* retitling windows is a memory leak because it doesn't know when to free up the window title. it will only leak if your window title changes a lot.
* `libnice-gstreamer1` (in fedora) is required in addition to the rest of the gstreamer plugins, esp if your webrtc element does not link (obscure error indeed).
* SDL2 bindings are included in streamerd and hyperwarp partially through a shared module and fully through the `sdl2-sys-lite` package. See the [license](https://www.libsdl.org/license.php) for more info.
* Thanks to the [libTAS](https://github.com/clementgallet/libTAS/) project for showing how to hook games in more complex scenarios. Their work on hooking SDL dynapi is invaluable.
* [selkies-gstreamer](https://github.com/selkies-project/selkies-gstreamer) has a very neat codebase for gstreamer, and I thank the contributors for their work. Their project has various performant optimizations that are also being used here.