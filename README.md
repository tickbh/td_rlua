## td_rlua

This library is a high-level binding for Lua 5.3. You don't have access to the Lua stack, all you can do is read/write variables (including callbacks) and execute Lua code.

### How to install it?

Add this to the `Cargo.toml` file of your project

```toml
[dependencies]
td_rlua = "0.1.0"
```

### How to use it?

```rust
extern crate td_rlua;
use td_rlua::Lua;
```

The `Lua` struct is the main element of this library. It represents a context in which you can execute Lua code.

```rust
let mut lua = Lua::new();     // mutable is mandatory
```

#### Reading and writing variables

```rust
lua.set("x", 2);
let _: () = lua.exec_string("x = x + 1").unwrap();
let x: i32 = lua.query("x").unwrap();
assert_eq!(x, 3);
```

Reading and writing global variables of the Lua context can be done with `set` and `query`.
The `query` function returns an `Option<T>` and does a copy of the value.

The base types that can be read and written are: `i8`, `i16`, `i32`, `u8`, `u16`, `u32`, `f32`, `f64`, `bool`, `String`. `&str` can be written but not read.

If you wish so, you can also add other types by implementing the `LuaPush` and `LuaRead` traits.

#### Executing Lua

```rust
let x: u32 = lua.exec_string("return 6 * 2;").unwrap();    // equals 12
```

The `exec_string` function takes a `&str` and returns a `Option<T>` where `T: LuaRead`.

#### Writing functions

In order to write a function, you must wrap it around `td_rlua::functionX` where `X` is the number of parameters. This is for the moment a limitation of Rust's inferrence system.

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

lua.set("add", td_rlua::function2(add));
let _: () = lua.exec_string("c = add(2, 4)").unwrap();   // calls the `add` function above
let c: i32 = lua.query("c").unwrap();
assert_eq!(c, 6);
```

In Lua, functions are exactly like regular variables.

You can write regular functions as well as closures:

```rust
lua.set("mul", td_rlua::function2(|a: i32, b: i32| a * b));
```

Note that the lifetime of the Lua context must be equal to or shorter than the lifetime of closures. This is enforced at compile-time.

```rust
let mut a = 5i;

{
    let mut lua = Lua::new();

    lua.set("inc", || a += 1);    // borrows 'a'
    for i in (0 .. 15) {
        let _: () = lua.exec_string("inc()").unwrap();
    }
} // unborrows `a`

assert_eq!(a, 20)
```

##### Error handling

```rust
extern "C" fn error_handle(lua: *mut c_lua::lua_State) -> libc::c_int {
    let err = unsafe { c_lua::lua_tostring(lua, -1) };
    let err = unsafe { CStr::from_ptr(err) };
    let err = String::from_utf8(err.to_bytes().to_vec()).unwrap();
    println!("error:{}", err);
    0
}
lua.register("error_handle", error_handle);
```

Default in exec_string will call pcall, and set the error_function _G["error_handle"] so you can reset 'error_handle' function to you custom.

#### Manipulating Lua tables

Manipulating a Lua table can be done by reading a `LuaTable` object. This can be achieved easily by reading a `LuaTable` object.

```rust
let _:() = lua.exec_string("a = { 9, 8, 7 }").unwrap();
let mut table : LuaTable = lua.query("a").unwrap();

let x: i32 = table.query(2).unwrap();
assert_eq!(x, 8);

table.set(3, "hello");
let y: String = table.query(3).unwrap();
assert_eq!(y, "hello");

let z: i32 = table.query(1).unwrap();
assert_eq!(z, 9);
```

You can then iterate through the table with the `.iter()` function. Note that the value returned by the iterator is an `Option<(Key, Value)>`, the `Option` being empty when either the key or the value is not convertible to the requested type. The `filter_map` function (provided by the standard `Iterator` trait) is very useful when dealing with this.

```rust
let _:() = lua.exec_string("a = { 9, 8, 7 }").unwrap();
let mut table : LuaTable = lua.query("a").unwrap();
for _ in 0 .. 10 {
    let table_content: Vec<Option<(u32, u32)>> = table.iter().collect();
    assert_eq!(table_content, vec![ Some((1,9)), Some((2,8)), Some((3,7)) ]);
}
```

#### User data

When you expose functions to Lua, you may wish to read or write more elaborate objects. This is called a **user data**.

To do so, you should implement the `LuaPush` for your types.
This is usually done by redirecting the call to `userdata::push_userdata`.
it will operate the ref of object
if you use `userdata::push_userdata` the userdata will copy one time, for lua gc manager
if you use `userdata::push_lightuserdata` the userdata life manager by rust, so none copy will occup

```rust
#[derive(Clone, Debug)]
struct Foo {
    a : i32,
};

impl<'a> td_rlua::LuaPush for &'a mut  Foo {
    fn push_to_lua(self, lua: *mut c_lua::lua_State) -> i32 {
        td_rlua::userdata::push_userdata(self, lua, |_|{})
    }
}
impl<'a> td_rlua::LuaRead for &'a mut  Foo {
    fn lua_read_at_position(lua: *mut c_lua::lua_State, index: i32) -> Option<&'a mut Foo> {
        td_rlua::userdata::read_userdata(lua, index)
    }
}

let xx  = &mut Foo {
    a : 10,
};
lua.set("a", xx);
let get: &mut Foo = lua.query("a").unwrap();
assert!(get.a == 10);
get.a = 100;

let get: &mut Foo = lua.query("a").unwrap();
assert!(get.a == 100);
```
use lightuserdata you can change
```rust
impl<'a> td_rlua::LuaPush for &'a mut  Foo {
    fn push_to_lua(self, lua: *mut c_lua::lua_State) -> i32 {
        td_rlua::userdata::push_lightuserdata(self, lua, |_|{})
    }
}
```

custom lua call userdata need impl NewStruct
```rust
#[derive(Clone, Debug)]
struct TestLuaSturct {
    index : i32,
}

impl NewStruct for TestLuaSturct {
    fn new() -> TestLuaSturct {
        println!("new !!!!!!!!!!!!!!");
        TestLuaSturct {
            index : 19,
        }
    }

    fn name() -> &'static str {
        "TestLuaSturct"
    }
}

impl<'a> LuaRead for &'a mut TestLuaSturct {
    fn lua_read_at_position(lua: *mut c_lua::lua_State, index: i32) -> Option<&'a mut TestLuaSturct> {
        td_rlua::userdata::read_userdata(lua, index)
    }
}
```

now we can custom function

```rust
let mut lua = Lua::new();
lua.openlibs();
fn one_arg(obj : &mut TestLuaSturct) -> i32 { obj.index = 10; 5 };
fn two_arg(obj : &mut TestLuaSturct, index : i32) { obj.index = index;};

let mut value = td_rlua::LuaStruct::<TestLuaSturct>::new(lua.state());
value.create().def("one_arg", td_rlua::function1(one_arg)).def("two_arg", td_rlua::function2(two_arg));

let _ : Option<()> = lua.exec_string("x = TestLuaSturct()");
let val : Option<i32> = lua.exec_string("return x:one_arg()");
assert_eq!(val, Some(5));
let obj : Option<&mut TestLuaSturct> = lua.query("x");
assert_eq!(obj.unwrap().index, 10);
let val : Option<i32> = lua.exec_string("x:two_arg(121)");
assert_eq!(val, None);
let obj : Option<&mut TestLuaSturct> = lua.query("x");
assert_eq!(obj.unwrap().index, 121);

let obj : Option<&mut TestLuaSturct> = lua.exec_string("return TestLuaSturct()");
assert_eq!(obj.unwrap().index, 19);
```
### Contributing

Contributions are welcome!
