use std::ffi::{CStr, CString};
use std::mem;

use c_lua;
use c_lua::lua_State;
use libc;

use LuaRead;
use LuaPush;

macro_rules! integer_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { c_lua::lua_pushinteger(lua, self as c_lua::lua_Integer) };
                1
            }
        }

        impl LuaRead for $t {
            fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<$t> {
                let mut success = unsafe { mem::uninitialized() };
                let val = unsafe { c_lua::lua_tointegerx(lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);

integer_impl!(i8);
integer_impl!(i16);
integer_impl!(i32);
integer_impl!(u8);
integer_impl!(u16);
integer_impl!(u32);

macro_rules! numeric_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { c_lua::lua_pushnumber(lua, self as f64) };
                1
            }
        }

        impl LuaRead for $t {
            fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<$t> {
                let mut success = unsafe { mem::uninitialized() };
                let val = unsafe { c_lua::lua_tonumberx(lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);

numeric_impl!(f32);
numeric_impl!(f64);

impl LuaPush for String {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        let value = CString::new(&self[..]).unwrap();
        unsafe { c_lua::lua_pushstring(lua, value.as_ptr()) };
        1
    }
}

impl LuaRead for String {
    fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<String> {
        let mut size: libc::size_t = unsafe { mem::uninitialized() };
        let c_str_raw = unsafe { c_lua::lua_tolstring(lua, index, &mut size) };
        if c_str_raw.is_null() {
            return None;
        }

        let c_str = unsafe { CStr::from_ptr(c_str_raw) };
        let c_str = String::from_utf8(c_str.to_bytes().to_vec()).unwrap();

        Some(c_str)
    }
}

impl<'s> LuaPush for &'s str {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        let value = CString::new(&self[..]).unwrap();
        unsafe { c_lua::lua_pushstring(lua, value.as_ptr()) };
        1
    }
}

impl LuaPush for bool {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { c_lua::lua_pushboolean(lua, self.clone() as libc::c_int) };
        1
    }
}

impl LuaRead for bool {
    fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<bool> {
        if unsafe { c_lua::lua_isboolean(lua, index) } != true {
            return None;
        }

        Some(unsafe { c_lua::lua_toboolean(lua, index) != 0 })
    }
}

impl LuaPush for () {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { c_lua::lua_pushnil(lua) };
        1
    }
}

impl LuaRead for () {
    fn lua_read_at_position(_: *mut lua_State, _: i32) -> Option<()> {
        Some(())
    }
}
