extern crate td_rlua;

use td_rlua::{Lua, LuaTable};

#[test]
fn iterable() {
    let mut lua = Lua::new();

    let _:() = lua.exec_string("a = { 9, 8, 7 }").unwrap();

    let mut table : LuaTable = lua.query("a").unwrap();
    let mut counter = 0;

    for (key, value) in table.iter().filter_map(|e| e) {
        let _: u32 = key;
        let _: u32 = value;
        assert_eq!(key + value, 10);
        counter += 1;
    }

    assert_eq!(counter, 3);
}

#[test]
fn iterable_multipletimes() {
    let mut lua = Lua::new();

    let _:() = lua.exec_string("a = { 9, 8, 7 }").unwrap();

    let mut table : LuaTable = lua.query("a").unwrap();

    for _ in 0 .. 10 {
        let table_content: Vec<Option<(u32, u32)>> = table.iter().collect();
        assert_eq!(table_content, vec![ Some((1,9)), Some((2,8)), Some((3,7)) ]);
    }
}

#[test]
fn get_set() {
    let mut lua = Lua::new();

    let _:() = lua.exec_string("a = { 9, 8, 7 }").unwrap();
    let mut table : LuaTable = lua.query("a").unwrap();

    let x: i32 = table.query(2).unwrap();
    assert_eq!(x, 8);

    table.set(3, "hello");
    let y: String = table.query(3).unwrap();
    assert_eq!(y, "hello");

    let z: i32 = table.query(1).unwrap();
    assert_eq!(z, 9);
}

#[test]
fn table_over_table() {
    let mut lua = Lua::new();

    let _:() = lua.exec_string("a = { 10, { 8, 7 }, 6 }").unwrap();
    let mut table : LuaTable = lua.query("a").unwrap();

    let x: i32 = table.query(1).unwrap();
    assert_eq!(x, 10);

    {
        let mut subtable : LuaTable = table.query(2).unwrap();

        let y: i32 = subtable.query(1).unwrap();
        assert_eq!(y, 8);

        let z: i32 = subtable.query(2).unwrap();
        assert_eq!(z, 7);
    }

    let w: i32 = table.query(3).unwrap();
    assert_eq!(w, 6);
}

#[test]
fn metatable() {
    let mut lua = Lua::new();

    let _:() = lua.exec_string("a = { 9, 8, 7 }").unwrap();

    {
        let mut table : LuaTable = lua.query("a").unwrap();

        let mut metatable = table.get_or_create_metatable();
        fn handler() -> i32 { 5 };
        metatable.set("__add".to_string(), td_rlua::function0(handler));
    }

    let r: i32 = lua.exec_string("return a + a").unwrap();
    assert_eq!(r, 5);
}

#[test]
fn empty_array() {
    let mut lua = Lua::new();

    {
        let mut array = lua.empty_table("a");
        array.set("b", 3)
    }

    let mut table: LuaTable = lua.query("a").unwrap();
    assert!(3 == table.query("b").unwrap());
}
