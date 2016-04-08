extern crate td_rlua;
extern crate td_clua;
extern crate libc;
use td_rlua::Lua;
use td_clua::*;
use td_rlua::LuaTable;
use td_rlua::LuaPush;
use td_rlua::LuaRead;
use td_rlua::NewStruct;

use std::any::{Any, TypeId};
use std::ffi::{CStr, CString};
trait Get {
    fn get_owner(self) -> Self;
}

#[derive(Debug)]
struct Test {
    a : i32,
}

impl Get for Test {
    fn get_owner(self) -> Self {
        // *self
        Test {
            a : 10,
        }
    }
}


fn test<T : Get>(a : T) -> T {
    a.get_owner()
}

fn test_userdata(state : &mut Lua) {
    #[derive(Clone, Debug)]
    struct Foo {
        a : i32,
    };

    impl Drop for Foo {
        fn drop(&mut self) {
            println!("drop fooooooooooooooo a = {}", self.a);
        }
    }
    impl<'a> td_rlua::LuaPush for &'a mut  Foo {
        fn push_to_lua(self, lua: *mut lua_State) -> i32 {
            // 1
            // td_rlua::userdata::push_userdata(self, lua, | a : LuaTable |{})
            td_rlua::userdata::push_lightuserdata(self, lua, | a : LuaTable |{})
        }
    }
    impl<'a> td_rlua::LuaRead for &'a mut  Foo {
        fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<&'a mut Foo> {
            // td_rlua::userdata::read_userdata(lua, index)
            td_rlua::userdata::read_userdata(lua, index)
            // if value.is_none() {
            //     None
            // } else {
            //     Some(* value.unwrap())
            // }
            // None
        }
    }

    {
        let xx  = &mut Foo {
            a : 10,
        };
        println!("111111111111111111");
        state.set("a", xx);
        println!("2222222222222222");
        let get: &mut Foo = state.query("a").unwrap();
        println!("get {:?}", get);
        get.a = 100;
    }

    // drop(xx);
    let get: &mut Foo = state.query("a").unwrap();
    println!("get {:?}", get);
    get.a = 103;

    let get: &mut Foo = state.query("a").unwrap();
    println!("get {:?}", get);
    get.a = 105;

}

extern "C" fn load_func(lua: *mut td_clua::lua_State) -> libc::c_int {
    let path = unsafe { td_clua::lua_tostring(lua, -1) };
    let path = unsafe { CStr::from_ptr(path) };
    let path = String::from_utf8(path.to_bytes().to_vec()).unwrap();
    println!("path:{}", path);
    0
}

fn custom_struct() {
    
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

    impl Drop for TestLuaSturct {
       fn drop(&mut self) {
            println!("drop test_lua_struct");
        }
    }

    impl<'a> LuaRead for &'a mut TestLuaSturct {
        fn lua_read_at_position(lua: *mut td_clua::lua_State, index: i32) -> Option<&'a mut TestLuaSturct> {
            td_rlua::userdata::read_userdata(lua, index)
        }
    }

    let mut lua = Lua::new();
    lua.openlibs();
    lua.enable_hotfix();
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
}
fn main() {
    custom_struct();
    // let mut state = Lua::new();
    // state.openlibs();
    // state.add_lualoader(load_func);
    // let _: Option<()> = state.exec_string("require \"test\"");

    // let xx  = Test { a : 1 };
    // // let yy = xx.get_owner();
    // let zz : Test = test(xx);
    // println!("{:?}", zz);

    // state.set("xx", 5);
    // let index = CString::new("xx").unwrap();
    // let xx : Option<i32> = state.query("xx");
    // // unsafe { td_clua::lua_getglobal(state.state(), index.as_ptr()); }
    // // "xx".push_to_lua(state);
    // // unsafe { td_clua::lua_setglobal(state.state(), index.as_ptr()); }
    // // let xx : i32 = td_rlua::LuaRead::lua_read(state).ok().unwrap();
    // println!("xx = {:?}", xx);

    // let mut value : LuaTable = state.exec_string("return {xx=1, aa=2, bb=3};").unwrap();
    // let table_content: Vec<Option<(String, u32)>> = value.iter().collect();
    // println!("table_content {:?}", table_content);
    // let ret : i32 = value.query("xx").unwrap();
    // println!("xx = {:?}", ret);

    // let mut value : LuaTable = state.exec_string("return {1, 2, 3};").unwrap();
    // let table_content: Vec<Option<(u32, u32)>> = value.iter().collect();
    // println!("table_content {:?}", table_content);
    // let ret : i32 = value.query(1).unwrap();
    // println!("xx = {:?}", ret);

    // test_userdata(&mut state);

    // fn add(val1: i32, val2: i32) -> i32 { val1 + val2 };

    // state.set("add", td_rlua::functions::function2(add));

    // let val: i32 = state.exec_string("return add(3, 7)").unwrap();
    // println!("val {:?}", val);
    // assert_eq!(val, 10);
}
