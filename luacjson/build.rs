extern crate pkg_config;
extern crate gcc;

fn main() {
    gcc::Build::new()
        .file("cjson/fpconv.c")
        .file("cjson/lua_cjson.c")
        .file("cjson/strbuf.c")
        .include("cjson")
        .include("include")
        .compile("libluacjson.a");
}
