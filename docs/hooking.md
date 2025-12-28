# Hooking
## LD_PRELOAD
### Conflicts
Hyperwarp uses LD_PRELOAD to inject it's code into the target process. Some other processes like performance wrapper utilities may also use LD_PRELOAD thus causing conflicts when they delete Hyperwarp's hooks.
### Loading order
Hyperwarp actually needs to load two different libraries. 

* `libhyperpreglue.so` - small C lib to capture dynamic linking calls before our main Rust component takes over. Source in `preglue.c`.
* `libhyperwarphooker.so` - Main library to hook grpahical functionality at high level. Source is in the `hyperwarp` crate.
* `libhyperglue.so` - small C lib to hijack calls before `main` and call apporpriate Rust entrypoint while taking over dlsym calls. Source in `glue.c`.

Thus a LD_PRELOAD order of `libhyperpreglue.so:libhyperwarphooker.so:libhyperglue.so` is required.

