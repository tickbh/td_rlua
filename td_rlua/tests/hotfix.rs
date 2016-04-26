extern crate td_rlua;

use td_rlua::Lua;


#[test]
fn hotfix_table() {
    let mut lua = Lua::new();
    lua.openlibs();
    lua.enable_hotfix();
    let _ : Option<()> = lua.exec_string(r"
        local value = {1, 2}
        function get_a()
            return value[1]
        end

        function get_b()
            return value[2]
        end
        ");

    let val: i32 = lua.exec_string("return get_a()").unwrap();
    assert_eq!(val, 1);

    let val: i32 = lua.exec_string("return get_b()").unwrap();
    assert_eq!(val, 2);

    let _ = lua.exec_func2("hotfix", r"
        local value = {3, 4}
        function get_a()
            value[2] = 3
            return value[1]
        end

        function get_b()
            return value[2]
        end
        ", "hotfix");

    let val: i32 = lua.exec_string("return get_b()").unwrap();
    assert_eq!(val, 2);

    let val: i32 = lua.exec_string("return get_a()").unwrap();
    assert_eq!(val, 1);

    let val: i32 = lua.exec_string("return get_b()").unwrap();
    assert_eq!(val, 3);
}

#[test]
fn hotfix() {
    let mut lua = Lua::new();
    lua.openlibs();
    lua.enable_hotfix();
    let _ : Option<()> = lua.exec_string(r"
        local a = 1
        local b = 2
        function get_a()
            return a
        end

        function get_b()
            return b
        end
        ");

    let val: i32 = lua.exec_string("return get_a()").unwrap();
    assert_eq!(val, 1);

    let val: i32 = lua.exec_string("return get_b()").unwrap();
    assert_eq!(val, 2);

    let _ = lua.exec_func2("hotfix", r"
        local a = 3
        local b = 4
        function get_a()
            b = 3
            return a
        end

        function get_b()
            return b
        end
        ", "hotfix");

    let val: i32 = lua.exec_string("return get_b()").unwrap();
    assert_eq!(val, 2);

    let val: i32 = lua.exec_string("return get_a()").unwrap();
    assert_eq!(val, 1);

    let val: i32 = lua.exec_string("return get_b()").unwrap();
    assert_eq!(val, 3);
}


#[test]
fn hotfix_module() {
    let mut lua = Lua::new();
    lua.openlibs();
    lua.enable_hotfix();
    let _ : Option<()> = lua.exec_string(r"
        USER_D = {}
        local _ENV = USER_D
        local a = 1
        local b = 2
        function get_a()
            return a
        end

        function get_b()
            return b
        end
        ");

    let val: i32 = lua.exec_string("return USER_D.get_a()").unwrap();
    assert_eq!(val, 1);

    let val: i32 = lua.exec_string("return USER_D.get_b()").unwrap();
    assert_eq!(val, 2);

    let _ = lua.exec_func2("hotfix", r"
        USER_D = {}
        local _ENV = USER_D
        local a = 3
        local b = 4
        function get_a()
            b = 3
            return a
        end

        function get_b()
            return b
        end
        ", "hotfix");

    let val: i32 = lua.exec_string("return USER_D.get_b()").unwrap();
    assert_eq!(val, 2);

    let val: i32 = lua.exec_string("return USER_D.get_a()").unwrap();
    assert_eq!(val, 1);

    let val: i32 = lua.exec_string("return USER_D.get_b()").unwrap();
    assert_eq!(val, 3);
}