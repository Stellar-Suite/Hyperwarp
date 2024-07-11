# dirty script to generate the list of SDL2 functions to hook
PREFIX = "pub const DYNAPI_FUNCS: [&str; {}] = [\n"
FORMAT = "  \"{}\",\n"
SUFFIX = "\n];"

seen = set()
funcs = []
for line in open("libTAS_sdlhooks.h"):
    line = line.strip()
    if not line.startswith("SDL_") or not line.endswith(")") or not "(" in line:
        continue
    func = line[line.index("(") + 1:-1]
    if func in seen:
        continue
    seen.add(func)
    funcs.append(func)

print("Found", len(funcs), "functions")
# write to file
with open("src/utils/sdl2_dynapi.rs", "w") as f:
    f.write(PREFIX.format(len(funcs)))
    for func in funcs:
        f.write(FORMAT.format(func))
    f.write(SUFFIX)