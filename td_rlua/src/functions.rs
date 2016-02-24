use c_lua;
use libc;
use c_lua::lua_State;
use LuaRead;
use LuaPush;

use std::marker::PhantomData;
use std::mem;
use std::ptr;

macro_rules! impl_function {
    ($name:ident, $($p:ident),*) => (
        /// Wraps a type that implements `FnMut` so that it can be used by hlua.
        ///
        /// This is only needed because of a limitation in Rust's inferrence system.
        pub fn $name<Z, R $(, $p)*>(f: Z) -> Function<Z, ($($p,)*), R> where Z: FnMut($($p),*) -> R {
            Function {
                function: f,
                marker: PhantomData,
            }
        }
    )
}

impl_function!(function0,);
impl_function!(function1, A);
impl_function!(function2, A, B);
impl_function!(function3, A, B, C);
impl_function!(function4, A, B, C, D);
impl_function!(function5, A, B, C, D, E);
impl_function!(function6, A, B, C, D, E, F);
impl_function!(function7, A, B, C, D, E, F, G);
impl_function!(function8, A, B, C, D, E, F, G, H);
impl_function!(function9, A, B, C, D, E, F, G, H, I);
impl_function!(function10, A, B, C, D, E, F, G, H, I, J);

/// Opaque type containing a Rust function or closure.
pub struct Function<F, P, R> {
    function: F,
    marker: PhantomData<(P, R)>,
}

/// Trait implemented on `Function` to mimic `FnMut`.
pub trait FunctionExt<P> {
    type Output;

    fn call_mut(&mut self, params: P) -> Self::Output;
}

macro_rules! impl_function_ext {
    () => (
        impl<Z, R> FunctionExt<()> for Function<Z, (), R> where Z: FnMut() -> R {
            type Output = R;

            #[allow(non_snake_case)]
            fn call_mut(&mut self, _: ()) -> Self::Output {
                (self.function)()
            }
        }

        impl<Z, R> LuaPush for Function<Z, (), R>
                where Z: FnMut() -> R,
                      R: LuaPush + 'static
        {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe {
                    // pushing the function pointer as a userdata
                    let lua_data = c_lua::lua_newuserdata(lua, mem::size_of::<Z>() as libc::size_t);
                    let lua_data: *mut Z = mem::transmute(lua_data);
                    ptr::write(lua_data, self.function);

                    // pushing wrapper as a closure
                    let wrapper: extern fn(*mut c_lua::lua_State) -> libc::c_int = wrapper::<Self, _, R>;
                    c_lua::lua_pushcclosure(lua, wrapper, 1);
                    1
                }
            }
        }
    );

    ($($p:ident),+) => (
        impl<Z, R $(,$p)*> FunctionExt<($($p,)*)> for Function<Z, ($($p,)*), R> where Z: FnMut($($p),*) -> R {
            type Output = R;

            #[allow(non_snake_case)]
            fn call_mut(&mut self, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)($($p),*)
            }
        }

        impl<Z, R $(,$p: 'static)+> LuaPush for Function<Z, ($($p,)*), R>
                where Z: FnMut($($p),*) -> R,
                      ($($p,)*): LuaRead,
                      R: LuaPush + 'static
        {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe {
                    // pushing the function pointer as a userdata
                    let lua_data = c_lua::lua_newuserdata(lua, mem::size_of::<Z>() as libc::size_t);
                    let lua_data: *mut Z = mem::transmute(lua_data);
                    ptr::write(lua_data, self.function);

                    // pushing wrapper as a closure
                    let wrapper: extern fn(*mut c_lua::lua_State) -> libc::c_int = wrapper::<Self, _, R>;
                    c_lua::lua_pushcclosure(lua, wrapper, 1);
                    1
                }
            }
        }
    )
}

impl_function_ext!();
impl_function_ext!(A);
impl_function_ext!(A, B);
impl_function_ext!(A, B, C);
impl_function_ext!(A, B, C, D);
impl_function_ext!(A, B, C, D, E);
impl_function_ext!(A, B, C, D, E, F);
impl_function_ext!(A, B, C, D, E, F, G);
impl_function_ext!(A, B, C, D, E, F, G, H);
impl_function_ext!(A, B, C, D, E, F, G, H, I);
impl_function_ext!(A, B, C, D, E, F, G, H, I, J);

// this function is called when Lua wants to call one of our functions
extern fn wrapper<T, P, R>(lua: *mut c_lua::lua_State) -> libc::c_int
                           where T: FunctionExt<P, Output=R>,
                                 P: LuaRead + 'static,
                                 R: LuaPush
{
    // loading the object that we want to call from the Lua context
    let data_raw = unsafe { c_lua::lua_touserdata(lua, c_lua::lua_upvalueindex(1)) };
    let data: &mut T = unsafe { mem::transmute(data_raw) };

    // trying to read the arguments
    let arguments_count = unsafe { c_lua::lua_gettop(lua) } as i32;
    let args = match LuaRead::lua_read_at_position(lua, -arguments_count as libc::c_int) {      // TODO: what if the user has the wrong params?
        Some(a) => a,
        _ => {
            let err_msg = format!("wrong parameter types for callback function");
            err_msg.push_to_lua(lua);
            unsafe { c_lua::lua_error(lua); }
            unreachable!()
        },
        
    };

    let ret_value = data.call_mut(args);

    // pushing back the result of the function on the stack
    let nb = ret_value.push_to_lua(lua);
    nb as libc::c_int
}

