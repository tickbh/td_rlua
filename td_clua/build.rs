extern crate pkg_config;
extern crate gcc;

fn main() {
    let mut build = gcc::Config::new();

        build.file("lua/src/lapi.c")
        .file("lua/src/lcode.c")
        .file("lua/src/lctype.c")
        .file("lua/src/ldebug.c")
        .file("lua/src/ldo.c")
        .file("lua/src/ldump.c")
        .file("lua/src/lfunc.c")
        .file("lua/src/lgc.c")
        .file("lua/src/llex.c")
        .file("lua/src/lmem.c")
        .file("lua/src/lobject.c")
        .file("lua/src/lopcodes.c")
        .file("lua/src/lparser.c")
        .file("lua/src/lstate.c")
        .file("lua/src/lstring.c")
        .file("lua/src/ltable.c")
        .file("lua/src/ltm.c")
        .file("lua/src/lundump.c")
        .file("lua/src/lvm.c")
        .file("lua/src/lzio.c")
        .file("lua/src/lauxlib.c")
        .file("lua/src/lbaselib.c")
        .file("lua/src/lbitlib.c")
        .file("lua/src/lcorolib.c")
        .file("lua/src/ldblib.c")
        .file("lua/src/liolib.c")
        .file("lua/src/lmathlib.c")
        .file("lua/src/loslib.c")
        .file("lua/src/lstrlib.c")
        .file("lua/src/ltablib.c")
        .file("lua/src/loadlib.c")
        .file("lua/src/linit.c")
        .file("lua/src/lutf8lib.c")
        .define("LUA_COMPAT_ALL", None)
        .define("LUA_COMPAT_MODULE", None)
        .define("LUA_COMPAT_BITLIB", None)
        .define("LUA_COMPAT_LOADSTRING", None);

    if cfg!(windows) {
        build.define("LUA_USE_WINDOWS", "1");
    }
    if cfg!(unix) {
        build.define("LUA_USE_LINUX", "1");
    }
    if cfg!(macos) {
        build.define("LUA_USE_MACOSX", "1");
    }
        
        build.include("lua/src")
        .compile("liblua.a");
}
