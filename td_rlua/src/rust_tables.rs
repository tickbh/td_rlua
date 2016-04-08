use td_clua;
use td_clua::lua_State;

use LuaPush;
use LuaRead;
use LuaTable;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

fn push_iter<V, I>(lua: *mut lua_State, iterator: I) -> i32
                      where V: LuaPush, I: Iterator<Item=V>
{
    // creating empty table
    unsafe { td_clua::lua_newtable(lua) };

    for (elem, index) in iterator.zip((1 ..)) {
        let size = elem.push_to_lua(lua);

        match size {
            0 => continue,
            1 => {
                let index = index as u32;
                index.push_to_lua(lua);
                unsafe { td_clua::lua_insert(lua, -2) }
                unsafe { td_clua::lua_settable(lua, -3) }
            },
            2 => unsafe { td_clua::lua_settable(lua, -3) },
            _ => unreachable!()
        }
    }

    1
}

fn push_rec_iter<V, I>(lua: *mut lua_State, iterator: I) -> i32
                          where V: LuaPush, I: Iterator<Item=V>
{
    let (nrec, _) = iterator.size_hint();

    // creating empty table with pre-allocated non-array elements
    unsafe { td_clua::lua_createtable(lua, 0, nrec as i32) };

    for elem in iterator {
        let size = elem.push_to_lua(lua);

        match size {
            0 => continue,
            2 => unsafe { td_clua::lua_settable(lua, -3) },
            _ => unreachable!()
        }
    }

    1
}

impl<T> LuaPush for Vec<T> where T: LuaPush {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        push_iter(lua, self.into_iter())
    }
}

impl<'a, T> LuaPush for &'a [T] where T: Clone + LuaPush {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        push_iter(lua, self.iter().map(|e| e.clone()))
    }
}

impl<K, V> LuaPush for HashMap<K, V> where K: LuaPush + Eq + Hash,
                                              V: LuaPush
{
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        push_rec_iter(lua, self.into_iter())
    }
}

impl<K> LuaPush for HashSet<K> where K: LuaPush + Eq + Hash
{
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        use std::iter;
        push_rec_iter(lua, self.into_iter().zip(iter::repeat(true)))
    }
}

impl<T> LuaRead for Vec<T> where T : LuaRead {
    fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<Vec<T>> {
        let mut lua_table : LuaTable = unwrap_or!(LuaRead::lua_read_at_position(lua, index), return None);
        let mut result = vec![];
        let len = lua_table.table_len();
        for i in 1 .. (len + 1) {
            let val : T = unwrap_or!(lua_table.query(i), return None);
            result.push(val);
        }
        Some(result)
    }
}