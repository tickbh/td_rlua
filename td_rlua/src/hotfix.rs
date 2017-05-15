use super::Lua;

// hot fix mod

///in runtime call hotfix func(reload code) or hotfix_file func(reload file)
///we will keep the old data but function, but hotfix not support change name,
///if we add new upvalue, it'a also support
///so after hotfix, the function is new and the data is old, so we success hotfix
pub fn load_hot_fix(lua: &mut Lua) {

    let func = r"
        function hotfix(chunk, check_name)
            check_name = check_name or 'hotfix'
            local env = {}
            setmetatable(env, { __index = _G })
            local _ENV = env
            local f, err = load(chunk, check_name,  't', env)
            assert(f,err)
            local ok, err = pcall(f)
            assert(ok,err)

            local protection = {
                setmetatable = true,
                pairs = true,
                ipairs = true,
                next = true,
                require = true,
                _ENV = true,
            }
            local visited_sig = {}
            function update_table(env_t, g_t, name, deep)
                if protection[env_t] or protection[g_t] then return end
                if env_t == g_t then return end
                local signature = tostring(g_t)..tostring(env_t)
                if visited_sig[signature] then return end
                visited_sig[signature] = true
                for name, value in pairs(env_t) do
                    local old_value = g_t[name]
                    if type(value) == type(old_value) then
                        if type(value) == 'function' then
                            update_func(value, old_value, name, deep..'  '..name..'  ')
                            g_t[name] = value
                        elseif type(value) == 'table' then
                            update_table(value, old_value, name, deep..'  '..name..'  ')
                        end
                    else
                        g_t[name] = value
                    end
                end

                local old_meta = debug.getmetatable(g_t)
                local new_meta = debug.getmetatable(env_t)
                if type(old_meta) == 'table' and type(new_meta) == 'table' then
                    update_table(new_meta, old_meta, name..'s Meta', deep..'  '..name..'s Meta'..'  ' )
                end
            end

            function update_func(env_f, g_f, name, deep)
                local signature = tostring(env_f)..tostring(g_f)
                if visited_sig[signature] then return end
                visited_sig[signature] = true
                local old_upvalue_map = {}
                for i = 1, math.huge do
                    local name, value = debug.getupvalue(g_f, i)
                    if not name then break end
                    old_upvalue_map[name] = value
                end

                for i = 1, math.huge do
                    local name, value = debug.getupvalue(env_f, i)
                    if not name then break end
                    local old_value = old_upvalue_map[name]
                    if old_value then
                        if type(old_value) ~= type(value) then
                            debug.setupvalue(env_f, i, old_value)
                        elseif type(old_value) == 'function' then
                            update_func(value, old_value, name, deep..'  '..name..'  ')
                        elseif type(old_value) == 'table' then
                            update_table(value, old_value, name, deep..'  '..name..'  ')
                            debug.setupvalue(env_f, i, old_value)
                        else
                            debug.setupvalue(env_f, i, old_value)
                        end
                    end
                end
            end
            
            for name,value in pairs(env) do
                local g_value = _G[name]
                if type(g_value) ~= type(value) then
                    _G[name] = value
                elseif type(value) == 'function' then
                    update_func(value, g_value, name, 'G'..'  ')
                    _G[name] = value
                elseif type(value) == 'table' then
                    update_table(value, g_value, name, 'G'..'  ')
                end
            end
            return 0
        end

        function hotfix_file(name)
            local file_str
            local fp = io.open(name)
            if fp then
                file_str = fp:read('*all')
                io.close(fp)
            end

            if not file_str then
                return -1
            end
            return hotfix(file_str, name)
    end";
    let _: Option<()> = lua.exec_string(func);
}
