# Hyperwarp
Disclaimer: THIS CODE IS EXTREMELY CURSED. NO QUALITY GUARANTEE IS PROVIDED. USE AT YOUR OWN RISK.

Please do not open PRs at the time since a lot of things are constantly changing and there is like 0 stability guarantees.

## notes
* retitling windows is a memory leak because it doesn't know when to free up the window title. it will only leak if your window title changes a lot.
* `libnice-gstreamer1` (in fedora) is required in addition to the rest of the gstreamer plugins, esp if your webrtc element does not link (obscure error indeed).
* SDL2 bindings are included in streamerd and hyperwarp but not nesscarily used. See the [license](https://www.libsdl.org/license.php) for more info.