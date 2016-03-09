extern crate c_lua;
extern crate libc;

use std::borrow::Borrow;
use std::ffi::{CStr, CString};

pub mod values;
pub mod lua_tables;
pub mod functions;
pub mod userdata;
pub mod tuples;
pub mod rust_tables;

pub use c_lua::*;
pub use functions::{function0, function1, function2, function3, function4, function5, function6, function7, function8, function9, function10, Function};
pub use userdata::{push_userdata, push_lightuserdata, read_userdata, LuaStruct, NewStruct};
pub use lua_tables::LuaTable;
#[allow(raw_pointer_derive)]
pub struct Lua {
    lua: *mut lua_State,
    own: bool,
}

macro_rules! impl_exec_func {
    ($name:ident, $($p:ident),*) => (
        #[allow(non_snake_case)]
        pub fn $name<Z, $($p),*>(&mut self, func_name : Z, $($p : $p, )*) -> i32 where Z: Borrow<str>, $($p : LuaPush),* {
            let func_name = CString::new(func_name.borrow()).unwrap();
            unsafe {
                let state = self.state();
                let error = CString::new("error_handle").unwrap();
                c_lua::lua_getglobal(state, error.as_ptr());
                c_lua::lua_getglobal(state, func_name.as_ptr());

                let mut index = 0;

                $(
                    $p.push_to_lua(self.state());
                    index += 1;
                )*

                let success = c_lua::lua_pcall(state, index, 0, -1 * index - 2);
                if success != 0 {
                    c_lua::lua_pop(state, 1);
                }
                success
                // 
                // LuaRead::lua_read(state)
            }
        }
    )
}

// TODO add lua require load func

impl Lua {
    /// Builds a new Lua context.
    ///
    /// # Panic
    ///
    /// The function panics if the underlying call to `luaL_newstate` fails
    /// (which indicates lack of memory).
    pub fn new() -> Lua {
        let lua = unsafe { c_lua::luaL_newstate() };
        if lua.is_null() {
            panic!("lua_newstate failed");
        }

        // called whenever lua encounters an unexpected error.
        extern "C" fn panic(lua: *mut c_lua::lua_State) -> libc::c_int {
            let err = unsafe { c_lua::lua_tostring(lua, -1) };
            let err = unsafe { CStr::from_ptr(err) };
            let err = String::from_utf8(err.to_bytes().to_vec()).unwrap();
            panic!("PANIC: unprotected error in call to Lua API ({})\n", err);
        }

        extern "C" fn error_handle(lua: *mut c_lua::lua_State) -> libc::c_int {
            let err = unsafe { c_lua::lua_tostring(lua, -1) };
            let err = unsafe { CStr::from_ptr(err) };
            let err = String::from_utf8(err.to_bytes().to_vec()).unwrap();
            println!("error:{}", err);
            0
        }

        unsafe { c_lua::lua_atpanic(lua, panic) };
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

    pub fn register<I>(&mut self, index : I, func : extern "C" fn(*mut c_lua::lua_State) -> libc::c_int) -> i32
                    where I: Borrow<str>
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { c_lua::lua_register(self.state(), index.as_ptr(), func) };
        0
    }

    /// Opens all standard Lua libraries.
    /// This is done by calling `luaL_openlibs`.
    pub fn openlibs(&mut self) {
        unsafe { c_lua::luaL_openlibs(self.lua) }
    }

    /// Reads the value of a global variable.
    pub fn query<'l, V, I>(&'l mut self, index: I) -> Option<V>
                         where I: Borrow<str>, V: LuaRead
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { c_lua::lua_getglobal(self.lua, index.as_ptr()); }
        LuaRead::lua_read(self.state())
    }

    /// Modifies the value of a global variable.
    pub fn set<I, V>(&mut self, index: I, value: V)
                         where I: Borrow<str>, for<'a> V: LuaPush
    {
        let index = CString::new(index.borrow()).unwrap();
        value.push_to_lua(self.state());
        unsafe { c_lua::lua_setglobal(self.lua, index.as_ptr()); }
    }

    pub fn exec_string<'a, I, R>(&'a mut self, index : I) -> Option<R>
                            where I: Borrow<str>, R : LuaRead
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe {
            let state = self.state();
            let error = CString::new("error_handle").unwrap();
            c_lua::lua_getglobal(state, error.as_ptr());
            c_lua::luaL_loadstring(state, index.as_ptr());
            let success = c_lua::lua_pcall(state, 0, 1, -2);
            if success != 0 {
                c_lua::lua_pop(state, 1);
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
            c_lua::lua_newtable(self.state());
            c_lua::lua_setglobal(self.state(), index2.as_ptr()); 
        }
        self.query(index).unwrap()
    }

    pub fn add_lualoader(&mut self, func : extern "C" fn(*mut c_lua::lua_State) -> libc::c_int) -> i32 {
        let state = self.state();
        unsafe {
            let package = CString::new("package").unwrap();
            let searchers = CString::new("searchers").unwrap();
            c_lua::lua_getglobal(state, package.as_ptr());
            c_lua::lua_getfield(state, -1, searchers.as_ptr());
            c_lua::lua_pushcfunction(state, func);
            let mut i = (c_lua::lua_rawlen(state, -2) + 1) as i32;
            while i > 2 {
                c_lua::lua_rawgeti(state, -2, i - 1);                               
                c_lua::lua_rawseti(state, -3, i);
                i = i - 1;
            }
            c_lua::lua_rawseti(state, -2, 2);                                        
            // set loaders into package
            c_lua::lua_setfield(state, -2, searchers.as_ptr());                               
            c_lua::lua_pop(state, 1);
        }
        0
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
    fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<Self>;
}

impl Drop for Lua {
    fn drop(&mut self) {
        if self.own {
            unsafe { c_lua::lua_close(self.lua) }
        }
    }
}

