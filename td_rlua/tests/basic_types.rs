extern crate td_rlua;

use td_rlua::Lua;

#[test]
fn read_i32s() {
    let mut lua = Lua::new();

    lua.set("a", 2);

    let x: i32 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(x, 2);

    let y: i8 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(y, 2);

    let z: i16 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(z, 2);

    let w: i32 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(w, 2);

    let a: u32 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(a, 2);

    let b: u8 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(b, 2);

    let c: u16 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(c, 2);

    let d : () = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(d, ());
}

#[test]
fn write_i32s() {
    // TODO: 

    let mut lua = Lua::new();

    lua.set("a", 2);
    let x: i32 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(x, 2);
}

#[test]
fn readwrite_floats() {
    let mut lua = Lua::new();

    lua.set("a", 2.51234 as f32);
    lua.set("b", 3.4123456789 as f64);

    let x: f32 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert!(x - 2.51234 < 0.000001);

    let y: f64 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert!(y - 2.51234 < 0.000001);

    let z: f32 = lua.query("b").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert!(z - 3.4123456789 < 0.000001);

    let w: f64 = lua.query("b").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert!(w - 3.4123456789 < 0.000001);
}

#[test]
fn readwrite_bools() {
    let mut lua = Lua::new();

    lua.set("a", true);
    lua.set("b", false);

    let x: bool = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(x, true);

    let y: bool = lua.query("b").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(y, false);
}

#[test]
fn readwrite_strings() {
    let mut lua = Lua::new();

    lua.set("a", "hello");
    lua.set("b", "hello".to_string());
    let unvaild = String::from_utf8_lossy(&[8, 0, 34, 0, 3, 0, 58, 0, 0, 0, 33, 0, 40, 0, 34, 0, 3, 0, 26, 0, 0, 0, 34, 0, 127, 0, 35, 0, 0, 0, 35, 0, 14]).to_string();
    lua.set("c", unvaild);

    let x: String = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(x, "hello");

    let y: String = lua.query("b").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(y, "hello");

    let z: String = lua.query("c").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(z, "UNVAILED STRING");
}

#[test]
fn i32_to_string() {
    let mut lua = Lua::new();

    lua.set("a", 2);

    let x: String = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(x, "2");
}

#[test]
fn string_to_i32() {
    let mut lua = Lua::new();

    lua.set("a", "2");
    lua.set("b", "aaa");

    let x: i32 = lua.query("a").unwrap();
    assert_eq!(lua.get_top(), 0);
    assert_eq!(x, 2);

    let y: Option<i32> = lua.query("b");
    assert_eq!(lua.get_top(), 0);
    assert!(y.is_none());
}
