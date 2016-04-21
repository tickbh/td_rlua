use std::marker::PhantomData;

use libc;

use td_clua::{self, lua_State};
use LuaPush;
use LuaRead;
use LuaGuard;
/// Represents a table stored in the Lua context.
///
/// Loading this type mutably borrows the Lua context.
pub struct LuaTable {
    table: *mut lua_State,
    pop : i32,
    index : i32,
}

impl LuaRead for LuaTable {
    fn lua_read_with_pop(lua: *mut lua_State, index: i32, pop: i32) -> Option<LuaTable> {
        if unsafe { td_clua::lua_istable(lua, index) } {
            for _ in 0 .. pop {
                unsafe { td_clua::lua_pushnil(lua); }
            }
            Some(LuaTable { table: lua, pop : pop, index : index })
        } else {
            None
        }
    }
}

impl Drop for LuaTable {
    fn drop(&mut self) {
        if self.pop != 0 {
            unsafe { td_clua::lua_pop(self.table, self.pop); };
            self.pop = 0;
        }
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

    /// Iterates over the elements inside the table.
    pub fn iter<K, V>(&mut self) -> LuaTableIterator<K, V> {
        unsafe { td_clua::lua_pushnil(self.table) };

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
        index.push_to_lua(self.table);
        unsafe { td_clua::lua_gettable(self.table, self.index - 1); }
        let _guard = LuaGuard::new(self.table, 1);
        LuaRead::lua_read_with_pop(self.table, -1, 1)
    }

    /// Inserts or modifies an elements of the table.
    pub fn set<I, V>(&mut self, index: I, value: V)
                         where I: LuaPush,
                               V: LuaPush
    {
        index.push_to_lua(self.table);
        value.push_to_lua(self.table);
        unsafe { td_clua::lua_settable(self.table, self.index - 2); }
    }

    /// Inserts or modifies an elements of the table.
    pub fn register<I>(&mut self, index: I, func : extern "C" fn(*mut lua_State) -> libc::c_int)
                         where I: LuaPush
    {
        index.push_to_lua(self.table);
        unsafe {
            td_clua::lua_pushcfunction(self.table, func);
            td_clua::lua_settable(self.table, self.index - 2);
        }
    }


    // /// Inserts an empty table, then loads it.
    pub fn empty_table<I>(&mut self, index: I) -> LuaTable
                              where I: LuaPush + Clone
    {
        index.clone().push_to_lua(self.table);
        unsafe { 
            td_clua::lua_newtable(self.table);
            td_clua::lua_settable(self.table, self.index - 2); 
        }
        self.query(index).unwrap()
    }

    pub fn table_len(&mut self) -> usize {
        unsafe {
            td_clua::lua_rawlen(self.table, self.index)
        }
    }

    // /// Obtains or create the metatable of the table.
    pub fn get_or_create_metatable(&mut self) -> LuaTable {
        let result = unsafe { td_clua::lua_getmetatable(self.table, self.index) };

        if result == 0 {
            unsafe {
                td_clua::lua_newtable(self.table);
                td_clua::lua_setmetatable(self.table, -2);
                let r = td_clua::lua_getmetatable(self.table, self.index);
                assert!(r != 0);
            }
        }

        LuaTable {
            table: self.table,
            pop: 1,
            index : -1,
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
        if unsafe { !td_clua::lua_istable(state, -2) || td_clua::lua_next(state, -2) == 0 } {
            self.finished = true;
            return None;
        }

        let key = LuaRead::lua_read_at_position(state, -2);
        let value = LuaRead::lua_read_at_position(state, -1);
        // removing the value, leaving only the key on the top of the stack
        unsafe { td_clua::lua_pop(state, 1) };

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
            unsafe { td_clua::lua_pop(self.table.table, 1) }
        }
    }
}
