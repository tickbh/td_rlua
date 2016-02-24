use c_lua;
use c_lua::lua_State;

use LuaPush;
use LuaRead;
use std::marker::PhantomData;
/// Represents a table stored in the Lua context.
///
/// Loading this type mutably borrows the Lua context.
pub struct LuaTable {
    table: *mut lua_State,
    top : i32,
}

impl LuaRead for LuaTable {
    fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<LuaTable> {
        assert!(index == -1);   // FIXME: not sure if it's working
        if unsafe { c_lua::lua_istable(lua, index) } {
            Some(LuaTable { table: lua, top : 0 })
        } else {
            None
        }
    }
}

impl Drop for LuaTable {
    fn drop(&mut self) {
        self.clear_top();
    }
}

/// Iterator that enumerates the content of a Lua table.
// while the LuaTableIterator is active, the current key is constantly pushed over the table
pub struct LuaTableIterator<'t, K, V> {
    table: &'t mut LuaTable,
    finished: bool,     // if true, the key is not on the stack anymore
    marker: PhantomData<(K, V)>,
}

impl LuaTable {
    /// Destroys the LuaTable and returns its inner Lua context. Useful when it takes Lua by value.
    pub fn into_inner(self) -> *mut lua_State {
        self.table
    }

    pub fn clear_top(&mut self) {
        if self.top != 0 {
            unsafe { c_lua::lua_pop(self.table, self.top); };
            self.top = 0;
        }
    }

    /// Iterates over the elements inside the table.
    pub fn iter<K, V>(&mut self) -> LuaTableIterator<K, V> {
        unsafe { c_lua::lua_pushnil(self.table) };

        LuaTableIterator {
            table: self,
            finished: false,
            marker: PhantomData,
        }
    }

    /// Loads a value in the table given its index.
    pub fn query<'a, R, I>(&'a mut self, index: I) -> Option<R>
                         where R: LuaRead,
                               I: LuaPush
    {
        self.clear_top();
        index.push_to_lua(self.table);
        unsafe { c_lua::lua_gettable(self.table, -2); }
        self.top = 1;
        LuaRead::lua_read(self.table)
    }

    /// Inserts or modifies an elements of the table.
    pub fn set<I, V>(&mut self, index: I, value: V)
                         where I: LuaPush,
                               V: LuaPush
    {
        self.clear_top();
        index.push_to_lua(self.table);
        value.push_to_lua(self.table);
        unsafe { c_lua::lua_settable(self.table, -3); }
    }

    // /// Inserts an empty table, then loads it.
    pub fn empty_table<I>(&mut self, index: I) -> LuaTable
                              where I: LuaPush + Clone
    {
        self.clear_top();
        index.clone().push_to_lua(self.table);
        unsafe { 
            c_lua::lua_newtable(self.table);
            c_lua::lua_settable(self.table, -3); 
        }
        self.query(index).unwrap()
    }

    // /// Obtains or create the metatable of the table.
    pub fn get_or_create_metatable(&mut self) -> LuaTable {
        self.clear_top();
        let result = unsafe { c_lua::lua_getmetatable(self.table, -1) };

        if result == 0 {
            unsafe {
                c_lua::lua_newtable(self.table);
                c_lua::lua_setmetatable(self.table, -2);
                let r = c_lua::lua_getmetatable(self.table, -1);
                assert!(r != 0);
            }
        }

        LuaTable {
            table: self.table,
            top: 0,
        }
    }
}

impl<'t, K, V> Iterator for LuaTableIterator<'t, K, V>
                  where K: LuaRead + 'static,
                        V: LuaRead + 'static
{
    type Item = Option<(K, V)>;

    fn next(&mut self) -> Option<Option<(K,V)>> {
        if self.finished {
            return None;
        }
        let state = self.table.table;
        // this call pushes the next key and value on the stack
        if unsafe { !c_lua::lua_istable(state, -2) || c_lua::lua_next(state, -2) == 0 } {
            self.finished = true;
            return None;
        }

        let key = LuaRead::lua_read_at_position(state, -2);
        let value = LuaRead::lua_read_at_position(state, -1);
        // removing the value, leaving only the key on the top of the stack
        unsafe { c_lua::lua_pop(state, 1) };

        if key.is_none() || value.is_none() {
            Some(None)
        } else {
            Some(Some((key.unwrap(), value.unwrap())))
        }
    }
}

impl<'t, K, V> Drop for LuaTableIterator<'t, K, V> {
    fn drop(&mut self) {
        if !self.finished {
            unsafe { c_lua::lua_pop(self.table.table, 1) }
        }
    }
}
