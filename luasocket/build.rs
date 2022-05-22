extern crate pkg_config;
extern crate gcc;

fn main() {
    let mut build = gcc::Build::new();

    if cfg!(unix) {
        build.file("luasocket/src/usocket.c");
        build.file("luasocket/src/unix.c");
        build.file("luasocket/src/unixdgram.c");
        build.file("luasocket/src/unixstream.c");
    }

    if cfg!(windows) {
        build.file("luasocket/src/wsocket.c");
    }

        build.file("luasocket/src/auxiliar.c")
        .file("luasocket/src/buffer.c")
        .file("luasocket/src/compat.c")
        .file("luasocket/src/except.c")
        .file("luasocket/src/inet.c")
        .file("luasocket/src/io.c")
        .file("luasocket/src/luasocket.c")
        .file("luasocket/src/options.c")
        .file("luasocket/src/select.c")
        .file("luasocket/src/tcp.c")
        .file("luasocket/src/timeout.c")
        .file("luasocket/src/udp.c")
        .include("include")
        .include("luasocket/src");

    if cfg!(windows) {
        build.define("LUA_USE_WINDOWS", "1");
    }
    if cfg!(unix) {
        build.define("LUA_USE_LINUX", "1");
    }
    if cfg!(macos) {
        build.define("LUA_USE_MACOSX", "1");
    }
        
    build.compile("libluasocket.a");
}
