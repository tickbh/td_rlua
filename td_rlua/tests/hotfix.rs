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
fn hotfix_local_funcion() {
    let mut lua = Lua::new();
    lua.openlibs();
    lua.enable_hotfix();
    let _ : Option<()> = lua.exec_string(r"
        CACHE_D = {}
        local _ENV = CACHE_D

        timer_map = {}

        local function delete_timer(rid)
            local map_info = timer_map[rid]
            if map_info == nil then
                return
            end
            if is_valid_timer(map_info['timer_id']) then
                delete_timer(map_info['timer_id'])
            end
            timer_map[rid] = nil
        end

        local function load_user_callback(data)
            assert(data['rid'] ~= nil, 'callback rid must no empty')
            if data.is_redis then
                delete_timer(data['rid'])
            end
            load_data_from_db(data['rid'], load_user_callback)
        end

        function get_data(rid, callback, callback_arg) 
            load_data_from_db(rid, load_user_callback)
        end

        function load_data_from_db(rid, callback)
        end

        function do_test()
            return 0
        end
        ");

    let val: i32 = lua.exec_string("return CACHE_D.do_test()").unwrap();
    assert_eq!(val, 0);

    let _ = lua.exec_func2("hotfix", r"
        CACHE_D = {}
        local _ENV = CACHE_D

        timer_map = {}

        local function delete_timer(rid)
            local map_info = timer_map[rid]
            if map_info == nil then
                return
            end
            if is_valid_timer(map_info['timer_id']) then
                delete_timer(map_info['timer_id'])
            end
            timer_map[rid] = nil
        end

        local function load_user_callback(data)
            assert(data['rid'] ~= nil, 'callback rid must no empty')
            if data.is_redis then
                delete_timer(data['rid'])
            end
            load_data_from_db(data['rid'], load_user_callback)
        end

        function get_data(rid, callback, callback_arg) 
            load_data_from_db(rid, load_user_callback)
        end

        function load_data_from_db(rid, callback)
        end

        function do_test()
            return 1
        end
        ", "hotfix");


    let val: i32 = lua.exec_string("return CACHE_D.do_test()").unwrap();
    assert_eq!(val, 1);
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