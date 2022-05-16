extern crate td_rlua;

#[allow(improper_ctypes)]
extern "C" {
    pub fn luaopen_socket(L : *mut td_rlua::lua_State);
}
