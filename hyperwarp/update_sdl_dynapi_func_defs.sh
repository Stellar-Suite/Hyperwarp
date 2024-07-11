# libTAS is awesome because the file in SDL2 has too many macros and is not easy to parse
wget https://raw.githubusercontent.com/clementgallet/libTAS/master/src/library/sdl/sdlhooks.h -O libTAS_sdlhooks.h 
python parse_sdl_dynapi_headers.py