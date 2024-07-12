# dirty script to generate the list of SDL2 functions to hook
PREFIX = "pub const DYNAPI_FUNCS: [&str; {}] = [\n"
FORMAT = "  \"{}\",\n"
SUFFIX = "\n];"

seen = set()
funcs = []
for line in open("sdl2_dynapi.h"):
    line = line.strip()
    if not line.startswith("SDL_DYNAPI_PROC(") or not line.endswith(")"):
        continue
    func = line.split(",")[1]
    if func in seen:
        continue
    seen.add(func)
    funcs.append(func)

# should say 601
print("Found", len(funcs), "functions")
# write to file
with open("src/utils/sdl2_dynapi.rs", "w") as f:
    f.write(PREFIX.format(len(funcs)))
    for func in funcs:
        f.write(FORMAT.format(func))
    f.write(SUFFIX)
    
