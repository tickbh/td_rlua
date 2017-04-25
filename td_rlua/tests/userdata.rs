extern crate td_rlua;

use td_rlua::lua_State;
use td_rlua::Lua;
use td_rlua::LuaPush;
use td_rlua::LuaRead;
use td_rlua::NewStruct;

#[test]
fn readwrite() {
    #[derive(Clone)]
    struct Foo;
    impl<'a> LuaPush for &'a mut  Foo {
        fn push_to_lua(self, lua: *mut lua_State) -> i32 {
            td_rlua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<'a> LuaRead for &'a mut Foo {
        fn lua_read_with_pop(lua: *mut lua_State, index: i32, _pop: i32) -> Option<&'a mut Foo> {
            td_rlua::userdata::read_userdata(lua, index)
        }
    }

    let mut lua = Lua::new();

    lua.set("a", &mut Foo);
    let _: &mut Foo = lua.query("a").unwrap();
}

#[test]
fn destructor_called() {
    use std::sync::{Arc, Mutex};

    let called = Arc::new(Mutex::new(false));

    struct Foo {
        called: Arc<Mutex<bool>>
    }

    impl Drop for Foo {
        fn drop(&mut self) {
            let mut called = self.called.lock().unwrap();
            (*called) = true;
        }
    }

    impl<'a> LuaPush for &'a mut Foo {
        fn push_to_lua(self, lua: *mut lua_State) -> i32 {
            td_rlua::userdata::push_userdata(self, lua, |_|{})
        }
    }

    {
        let mut lua = Lua::new();
        lua.set("a", &mut Foo{called: called.clone()});
    }

    let locked = called.lock().unwrap();
    assert!(*locked);
}

#[test]
fn type_check() {
    #[derive(Clone)]
    struct Foo;
    impl<'a> LuaPush for &'a mut Foo {
        fn push_to_lua(self, lua: *mut lua_State) -> i32 {
            td_rlua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<'a> LuaRead for &'a mut Foo {
        fn lua_read_with_pop(lua: *mut lua_State, index: i32, _pop: i32) -> Option<&'a mut Foo> {
            td_rlua::userdata::read_userdata(lua, index)
        }
    }

    #[derive(Clone)]
    struct Bar;
    impl<'a> LuaPush for &'a mut Bar {
        fn push_to_lua(self, lua: *mut lua_State) -> i32 {
            td_rlua::userdata::push_userdata(self, lua, |_|{})
        }
    }
    impl<'a> LuaRead for &'a mut Bar {
        fn lua_read_with_pop(lua: *mut lua_State, index: i32, _pop: i32) -> Option<&'a mut Bar> {
            td_rlua::userdata::read_userdata(lua, index)
        }
    }

    let mut lua = Lua::new();

    lua.set("a", &mut Foo);
    
    let x: Option<&mut Bar> = lua.query("a");
    assert!(x.is_none())
}

#[test]
fn metatables() {
    #[derive(Clone)]
    struct Foo;
    impl<'a> LuaPush for &'a mut Foo {
        fn push_to_lua(self, lua: *mut lua_State) -> i32 {
            td_rlua::userdata::push_userdata(self, lua, |mut table| {
                table.set("__index".to_string(), vec![
                    // ("test".to_string(), td_rlua::function0(|| 5)),
                    ("test1".to_string(), td_rlua::function1(|a : i32| a)),
                ]);
            })
        }
    }

    let mut lua = Lua::new();

    lua.set("a", &mut Foo);

    let x: i32 = lua.exec_string("return a.test1(5)").unwrap();
    assert_eq!(x, 5);
}


#[test]
fn get_set_test() {
    let mut lua = Lua::new();
    #[derive(Clone, Debug)]
    struct Foo {
        a : i32,
    };

    impl td_rlua::LuaPush for Foo {
        fn push_to_lua(self, lua: *mut lua_State) -> i32 {
            let t = Box::into_raw(Box::new(self));
            unsafe {
                td_rlua::userdata::push_userdata::<Foo, _>(::std::mem::transmute(&mut *t), lua, |_|{})
            }
        }
    }
    impl<'a> td_rlua::LuaRead for &'a mut  Foo {
        fn lua_read_with_pop(lua: *mut lua_State, index: i32, _pop: i32) -> Option<&'a mut Foo> {
            td_rlua::userdata::read_userdata(lua, index)
        }
    }

    let xx = Foo {
        a : 10,
    };
    lua.set("a", xx);
    let get: &mut Foo = lua.query("a").unwrap();
    assert!(get.a == 10);
    get.a = 100;

    let get: &mut Foo = lua.query("a").unwrap();
    assert!(get.a == 100);
}


#[test]
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
        fn lua_read_with_pop(lua: *mut lua_State, index: i32, _pop: i32) -> Option<&'a mut TestLuaSturct> {
            td_rlua::userdata::read_userdata(lua, index)
        }
    }

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
}