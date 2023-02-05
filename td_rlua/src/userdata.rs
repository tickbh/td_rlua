use std::any::{Any, TypeId};
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::marker::PhantomData;
use std::boxed::Box;

use td_clua;
use td_clua::lua_State;
use libc;
use Lua;
use LuaPush;
use LuaRead;
use LuaTable;

// Called when an object inside Lua is being dropped.
#[inline]
extern "C" fn destructor_wrapper<T>(lua: *mut td_clua::lua_State) -> libc::c_int {
    unsafe {
        let obj = td_clua::lua_touserdata(lua, -1);
        ptr::drop_in_place(obj as *mut T);
        0
    }
}

extern "C" fn constructor_wrapper<T>(lua: *mut td_clua::lua_State) -> libc::c_int
    where T: NewStruct + Any {
    let t = T::new();
    let lua_data_raw = unsafe {
        td_clua::lua_newuserdata(lua, mem::size_of::<T>() as libc::size_t)
    };
    unsafe {
        ptr::write(lua_data_raw as *mut _, t);
    }
    let typeid = CString::new(T::name()).unwrap();
    unsafe {
        td_clua::lua_getglobal(lua, typeid.as_ptr());
        td_clua::lua_setmetatable(lua, -2);
    }
    1
}

// constructor direct create light object,
// in rust we alloc the memory, avoid copy the memory
// in lua we get the object, we must free the memory
extern "C" fn constructor_light_wrapper<T>(lua: *mut td_clua::lua_State) -> libc::c_int 
    where T: NewStruct + Any {
    let t = Box::into_raw(Box::new(T::new()));
    push_lightuserdata(unsafe { &mut *t }, lua, |_| {});
    let typeid = CString::new(T::name()).unwrap();
    unsafe {
        td_clua::lua_getglobal(lua, typeid.as_ptr());
        td_clua::lua_setmetatable(lua, -2);
    }
    1
}

/// Pushes an object as a user data.
///
/// In Lua, a user data is anything that is not recognized by Lua. When the script attempts to
/// copy a user data, instead only a reference to the data is copied.
///
/// The way a Lua script can use the user data depends on the content of the **metatable**, which
/// is a Lua table linked to the object.
///
/// # Arguments
///
///  - `metatable`: Function that fills the metatable of the object.
///
pub fn push_userdata<'a, T, F>(data: T,
                               lua: *mut td_clua::lua_State,
                               mut metatable: F)
                               -> i32
    where F: FnMut(LuaTable),
          T: 'a + Any
{
    let typeid = format!("{:?}", TypeId::of::<T>());
    let lua_data_raw = unsafe {
        td_clua::lua_newuserdata(lua, mem::size_of::<T>() as libc::size_t)
    };

    // creating a metatable
    unsafe {
        
        ptr::write(lua_data_raw as *mut _, data);

        td_clua::lua_newtable(lua);

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(lua);
        typeid.push_to_lua(lua);
        td_clua::lua_settable(lua, -3);

        // index "__gc" call the object's destructor
        {
            "__gc".push_to_lua(lua);

            td_clua::lua_pushcfunction(lua, destructor_wrapper::<T>);

            td_clua::lua_settable(lua, -3);
        }

        // calling the metatable closure
        {
            metatable(LuaRead::lua_read(lua).unwrap());
        }

        td_clua::lua_setmetatable(lua, -2);
    }

    1
}


/// Pushes an object as a user data.
///
/// In Lua, a user data is anything that is not recognized by Lua. When the script attempts to
/// copy a user data, instead only a reference to the data is copied.
///
/// The way a Lua script can use the user data depends on the content of the **metatable**, which
/// is a Lua table linked to the object.
///
/// # Arguments
///
///  - `metatable`: Function that fills the metatable of the object.
///
pub fn push_lightuserdata<'a, T, F>(data: &'a mut T,
                                    lua: *mut td_clua::lua_State,
                                    mut metatable: F)
                                    -> i32
    where F: FnMut(LuaTable),
          T: 'a + Any
{
    let typeid = format!("{:?}", TypeId::of::<T>());
    unsafe {
        td_clua::lua_pushlightuserdata(lua, mem::transmute(data));
    };

    // creating a metatable
    unsafe {

        td_clua::lua_newtable(lua);

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(lua);
        typeid.push_to_lua(lua);
        td_clua::lua_settable(lua, -3);

        // calling the metatable closure
        {
            metatable(LuaRead::lua_read(lua).unwrap());
        }

        td_clua::lua_setmetatable(lua, -2);
    }

    1
}

/// 
pub fn read_userdata<'t, 'c, T>(lua: *mut td_clua::lua_State, index: i32) -> Option<&'t mut T>
    where T: 'static + Any
{
    unsafe {
        let expected_typeid = format!("{:?}", TypeId::of::<T>());
        let data_ptr = td_clua::lua_touserdata(lua, index);
        if data_ptr.is_null() {
            return None;
        }
        if td_clua::lua_getmetatable(lua, index) == 0 {
            return None;
        }

        "__typeid".push_to_lua(lua);
        td_clua::lua_gettable(lua, -2);
        match <String as LuaRead>::lua_read(lua) {
            Some(ref val) if val == &expected_typeid => {}
            _ => {
                td_clua::lua_pop(lua, 2);
                return None;
            }
        }
        td_clua::lua_pop(lua, 2);
        Some(mem::transmute(data_ptr))
    }
}

pub trait NewStruct {
    fn new() -> Self;
    fn name() -> &'static str;
}

pub struct LuaStruct<T> {
    lua: *mut lua_State,
    light: bool,
    marker: PhantomData<T>,
}

impl<T> LuaStruct<T>
    where T: NewStruct + Any
{
    pub fn new(lua: *mut lua_State) -> LuaStruct<T> {
        LuaStruct {
            lua: lua,
            light: false,
            marker: PhantomData,
        }
    }

    pub fn new_light(lua: *mut lua_State) -> LuaStruct<T> {
        LuaStruct {
            lua: lua,
            light: true,
            marker: PhantomData,
        }
    }

    pub fn ensure_matetable(&mut self) {
        let name = T::name();
        let mut lua = Lua::from_existing_state(self.lua, false);

        match lua.query::<LuaTable, _>(name.clone()) {
            Some(_) => {}
            None => unsafe {
                td_clua::lua_newtable(self.lua);

                let typeid = format!("{:?}", TypeId::of::<T>());
                // index "__name" corresponds to the hash of the TypeId of T
                "__typeid".push_to_lua(self.lua);
                typeid.push_to_lua(self.lua);
                td_clua::lua_settable(self.lua, -3);

                // index "__gc" call the object's destructor
                if !self.light {
                    "__gc".push_to_lua(self.lua);
                    
                    td_clua::lua_pushcfunction(self.lua, destructor_wrapper::<T>);

                    td_clua::lua_settable(self.lua, -3);
                }

                "__index".push_to_lua(self.lua);
                td_clua::lua_newtable(self.lua);
                td_clua::lua_rawset(self.lua, -3);

                let name = CString::new(name).unwrap();
                td_clua::lua_setglobal(self.lua, name.as_ptr());
            },
        }
    }

    pub fn create(&mut self) -> &mut LuaStruct<T> {
        self.ensure_matetable();
        unsafe {
            let typeid = CString::new(T::name()).unwrap();
            td_clua::lua_getglobal(self.lua, typeid.as_ptr());
            if td_clua::lua_istable(self.lua, -1) {
                td_clua::lua_newtable(self.lua);
                "__call".push_to_lua(self.lua);

                if self.light {
                    td_clua::lua_pushcfunction(self.lua, constructor_light_wrapper::<T>);
                    td_clua::lua_settable(self.lua, -3);
                } else {
                    td_clua::lua_pushcfunction(self.lua, constructor_wrapper::<T>);
                    td_clua::lua_settable(self.lua, -3);
                }

                td_clua::lua_setmetatable(self.lua, -2);
            }
            td_clua::lua_pop(self.lua, 1);
        }
        self
    }

    pub fn def<P>(&mut self, name: &str, param: P) -> &mut LuaStruct<T>
        where P: LuaPush
    {
        let tname = T::name();
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(tname.clone()) {
            Some(mut table) => {
                match table.query::<LuaTable, _>("__index") {
                    Some(mut index) => {
                        index.set(name, param);
                    }
                    None => {
                        let mut index = table.empty_table("__index");
                        index.set(name, param);
                    }
                };
            }
            None => (),
        };
        self
    }


    pub fn register(&mut self,
                    name: &str,
                    func: extern "C" fn(*mut td_clua::lua_State) -> libc::c_int)
                    -> &mut LuaStruct<T> {
        let tname = T::name();
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(tname.clone()) {
            Some(mut table) => {
                match table.query::<LuaTable, _>("__index") {
                    Some(mut index) => {
                        index.register(name, func);
                    }
                    None => {
                        let mut index = table.empty_table("__index");
                        index.register(name, func);
                    }
                };
            }
            None => (),
        };
        self
    }
}
