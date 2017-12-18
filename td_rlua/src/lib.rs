extern crate td_clua;
extern crate libc;

use std::borrow::Borrow;
use std::ffi::{CStr, CString};
use std::io::prelude::*;
use std::fs::File;

macro_rules! unwrap_or {
    ($expr:expr, $or:expr) => (
        match $expr {
            Some(x) => x,
            None => { $or }
        }
    )
}

pub mod values;
pub mod lua_tables;
pub mod functions;
pub mod userdata;
pub mod tuples;
pub mod rust_tables;
mod hotfix;

pub use td_clua::*;
pub use functions::{function0, function1, function2, function3, function4, function5, function6, function7, function8, function9, function10, Function};
pub use userdata::{push_userdata, push_lightuserdata, read_userdata, LuaStruct, NewStruct};
pub use lua_tables::LuaTable;
pub struct Lua {
    lua: *mut lua_State,
    own: bool,
}


pub struct LuaGuard {
    pub lua: *mut lua_State,
    pub size: i32,
}

impl LuaGuard {

    pub fn forget(mut self) -> i32 {
        let size = self.size;
        self.size = 0;
        size
    }

    pub fn empty(&self) -> LuaGuard {
        LuaGuard {
            lua: self.lua,
            size: 0,
        }
    }

    pub fn new_empty(lua: *mut lua_State) -> LuaGuard {
        LuaGuard {
            lua: lua,
            size: 0,
        }
    }

    pub fn new(lua: *mut lua_State, size: i32) -> LuaGuard {
        LuaGuard {
            lua: lua,
            size: size,
        }
    }
}


macro_rules! impl_exec_func {
    ($name:ident, $($p:ident),*) => (
        #[allow(non_snake_case, unused_mut)]
        pub fn $name<Z, $($p),*>(&mut self, func_name : Z, $($p : $p, )*) -> i32 where Z: Borrow<str>, $($p : LuaPush),* {
            let func_name = CString::new(func_name.borrow()).unwrap();
            unsafe {
                let state = self.state();
                let error = CString::new("error_handle").unwrap();
                lua_getglobal(state, error.as_ptr());
                td_clua::lua_getglobal(state, func_name.as_ptr());

                let mut index = 0;
                $(
                    index += $p.push_to_lua(self.state());
                )*

                let success = td_clua::lua_pcall(state, index, 0, -1 * index - 2);
                if success != 0 {
                    td_clua::lua_pop(state, 1);
                }
                td_clua::lua_pop(state, 1);
                success
            }
        }
    )
}

impl Lua {
    /// Builds a new Lua context.
    ///
    /// # Panic
    ///
    /// The function panics if the underlying call to `luaL_newstate` fails
    /// (which indicates lack of memory).
    pub fn new() -> Lua {
        let lua = unsafe { td_clua::luaL_newstate() };
        if lua.is_null() {
            panic!("lua_newstate failed");
        }

        // called whenever lua encounters an unexpected error.
        extern "C" fn panic(lua: *mut td_clua::lua_State) -> libc::c_int {
            let err = unsafe { td_clua::lua_tostring(lua, -1) };
            let err = unsafe { CStr::from_ptr(err) };
            let err = String::from_utf8(err.to_bytes().to_vec()).unwrap();
            panic!("PANIC: unprotected error in call to Lua API ({})\n", err);
        }

        extern "C" fn error_handle(lua: *mut td_clua::lua_State) -> libc::c_int {
            let err = unsafe { td_clua::lua_tostring(lua, -1) };
            let err = unsafe { CStr::from_ptr(err) };
            let err = String::from_utf8(err.to_bytes().to_vec()).unwrap();
            println!("error:{}", err);
            0
        }

        unsafe { td_clua::lua_atpanic(lua, panic) };
        let mut lua = Lua {
            lua: lua,
            own: true,
        };
        lua.register("error_handle", error_handle);
        lua
    }

    pub fn state(&mut self) -> *mut lua_State {
        return self.lua;
    }

    pub fn clone(&mut self) -> Lua {
        Lua {
            lua : self.lua,
            own : false,
        }
    }

    pub fn set_own(&mut self, own: bool) {
        self.own = own;
    }

    /// Takes an existing `lua_State` and build a Lua object from it.
    ///
    /// # Arguments
    ///
    ///  * `close_at_the_end`: if true, lua_close will be called on the lua_State on the destructor
    pub fn from_existing_state(lua: *mut lua_State, close_at_the_end: bool) -> Lua {
        Lua {
            lua : lua,
            own: close_at_the_end,
        }
    }

    pub fn register<I>(&mut self, index : I, func : extern "C" fn(*mut td_clua::lua_State) -> libc::c_int) -> i32
                    where I: Borrow<str>
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { td_clua::lua_register(self.state(), index.as_ptr(), func) };
        0
    }

    /// Opens all standard Lua libraries.
    /// This is done by calling `luaL_openlibs`.
    pub fn openlibs(&mut self) {
        unsafe { td_clua::luaL_openlibs(self.lua) }
    }

    /// Reads the value of a global variable.
    pub fn query<'l, V, I>(&'l mut self, index: I) -> Option<V>
                         where I: Borrow<str>, V: LuaRead
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { td_clua::lua_getglobal(self.lua, index.as_ptr()); }
        let _guard = LuaGuard::new(self.lua, 1);
        LuaRead::lua_read_with_pop(self.state(), -1, 1)
    }

    /// Modifies the value of a global variable.
    pub fn set<I, V>(&mut self, index: I, value: V)
                         where I: Borrow<str>, for<'a> V: LuaPush
    {
        let index = CString::new(index.borrow()).unwrap();
        value.push_to_lua(self.state());
        unsafe { td_clua::lua_setglobal(self.lua, index.as_ptr()); }
    }

    pub fn exec_string<'a, I, R>(&'a mut self, index : I) -> Option<R>
                            where I: Borrow<str>, R : LuaRead
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe {
            let state = self.state();
            let error = CString::new("error_handle").unwrap();
            td_clua::lua_getglobal(state, error.as_ptr());
            td_clua::luaL_loadstring(state, index.as_ptr());
            let success = td_clua::lua_pcall(state, 0, 1, -2);
            if success != 0 {
                td_clua::lua_pop(state, 1);
                return None;
            }
            LuaRead::lua_read(state)
        }
    }

    pub fn exec_func<'a, I, R>(&'a mut self, index : I) -> Option<R>
                            where I: Borrow<str>, R : LuaRead
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe {
            let state = self.state();
            let error = CString::new("error_handle").unwrap();
            let top = td_clua::lua_gettop(state);
            td_clua::lua_getglobal(state, index.as_ptr());
            td_clua::lua_insert(state, -top - 1);
            td_clua::lua_getglobal(state, error.as_ptr());
            td_clua::lua_insert(state, -top - 2);
            let success = td_clua::lua_pcall(state, top, 1, -top-2);
            if success != 0 {
                td_clua::lua_pop(state, 1);
                return None;
            }
            LuaRead::lua_read(state)
        }
    }

    /// Inserts an empty table, then loads it.
    pub fn empty_table<I>(&mut self, index: I) -> LuaTable
                              where I: Borrow<str>
    {
        let index2 = CString::new(index.borrow()).unwrap();
        unsafe { 
            td_clua::lua_newtable(self.state());
            td_clua::lua_setglobal(self.state(), index2.as_ptr()); 
        }
        self.query(index).unwrap()
    }

    pub fn add_lualoader(&mut self, func : extern "C" fn(*mut td_clua::lua_State) -> libc::c_int) -> i32 {
        let state = self.state();
        unsafe {
            let package = CString::new("package").unwrap();
            let searchers = CString::new("searchers").unwrap();
            td_clua::lua_getglobal(state, package.as_ptr());
            td_clua::lua_getfield(state, -1, searchers.as_ptr());
            td_clua::lua_pushcfunction(state, func);
            let mut i = (td_clua::lua_rawlen(state, -2) + 1) as i32;
            while i > 2 {
                td_clua::lua_rawgeti(state, -2, i - 1);                               
                td_clua::lua_rawseti(state, -3, i);
                i = i - 1;
            }
            td_clua::lua_rawseti(state, -2, 2);                                        
            // set loaders into package
            td_clua::lua_setfield(state, -2, searchers.as_ptr());                               
            td_clua::lua_pop(state, 1);
        }
        0
    }

    pub fn load_file(&mut self, file_name: &str) -> i32 {
        let mut f = unwrap_or!(File::open(file_name).ok(), return 0);
        let mut buffer = Vec::new();
        let _ = unwrap_or!(f.read_to_end(&mut buffer).ok(), return 0);
        let mut name = file_name.to_string();
        let mut short_name = name.clone();
        let len = name.len();
        if len > 30 {
            short_name = name.drain((len - 30)..).collect();
        }

        let short_name = CString::new(short_name).unwrap();
        let ret = unsafe { td_clua::luaL_loadbuffer(self.state(), buffer.as_ptr() as *const i8, buffer.len(), short_name.as_ptr()) };
        if ret != 0 {
            let err_msg : String = unwrap_or!(LuaRead::lua_read(self.state()), return 0);
            let err_detail = CString::new(format!("error loading from file {} :\n\t{}", file_name, err_msg)).unwrap();
            unsafe { td_clua::luaL_error(self.state(), err_detail.as_ptr()); }
        }
        1
    }

    /// enable hotfix, can update the new func, and the old data will be keep and bind to the new func
    pub fn enable_hotfix(&mut self) {
        hotfix::load_hot_fix(self);
    }

    pub fn exec_gc(&mut self) -> i32 {
        unsafe { td_clua::lua_gc(self.state(), td_clua::LUA_GCCOLLECT, 0) as i32 } 
    }

    impl_exec_func!(exec_func0, );
    impl_exec_func!(exec_func1, A);
    impl_exec_func!(exec_func2, A, B);
    impl_exec_func!(exec_func3, A, B, C);
    impl_exec_func!(exec_func4, A, B, C, D);
    impl_exec_func!(exec_func5, A, B, C, D, E);
    impl_exec_func!(exec_func6, A, B, C, D, E, F);
    impl_exec_func!(exec_func7, A, B, C, D, E, F, G);
    impl_exec_func!(exec_func8, A, B, C, D, E, F, G, H);
    impl_exec_func!(exec_func9, A, B, C, D, E, F, G, H, I);
    impl_exec_func!(exec_func10, A, B, C, D, E, F, G, H, I, J);

}

/// Types that can be given to a Lua context, for example with `lua.set()` or as a return value
/// of a function.
pub trait LuaPush {
    /// Pushes the value on the top of the stack.
    ///
    /// Must return a guard representing the elements that have been pushed.
    ///
    /// You can implement this for any type you want by redirecting to call to
    /// another implementation (for example `5.push_to_lua`) or by calling
    /// `userdata::push_userdata`.
    fn push_to_lua(self, lua: *mut lua_State) -> i32;
}

/// Types that can be obtained from a Lua context.
///
/// Most types that implement `LuaPush` also implement `LuaRead`, but this is not always the case
/// (for example `&'static str` implements `LuaPush` but not `LuaRead`).
pub trait LuaRead: Sized {
    /// Reads the data from Lua.
    fn lua_read(lua: *mut lua_State) -> Option<Self> {
        LuaRead::lua_read_at_position(lua, -1)
    }

    /// Reads the data from Lua at a given position.
    fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<Self> {
        LuaRead::lua_read_with_pop(lua, index, 0)
    }

    /// Reads the data from Lua at a given position.
    fn lua_read_with_pop(lua: *mut lua_State, index: i32, pop: i32) -> Option<Self>;
}

impl Drop for Lua {
    fn drop(&mut self) {
        if self.own {
            unsafe { td_clua::lua_close(self.lua) }
        }
    }
}

impl Drop for LuaGuard {
    fn drop(&mut self) {
        if self.size != 0 {
            unsafe { td_clua::lua_pop(self.lua, self.size) }
        }
    }
}