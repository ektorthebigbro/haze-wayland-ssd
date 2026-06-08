// SPDX-License-Identifier: GPL-3.0
// Copyright (C) 2026 Haze Project

#![allow(non_camel_case_types)]

use glib::prelude::Cast;
use glib::subclass::prelude::*;
use glib::translate::IntoGlibPtr;
use glib::types::StaticType;
use glib::value::ToValue;
use glib::ParamSpecBuilderExt;
use libc::{c_char, c_int, c_void};
use once_cell::sync::{Lazy, OnceCell};
use std::cell::Cell;
use std::collections::HashSet;
use std::ffi::CStr;
use std::ptr;
use std::sync::{Mutex, MutexGuard};

const VERSION: &CStr = c"0.1.0";
const HOST_STATUS: &CStr = c"supported";
const MANAGER_VERSION: c_int = 2;
const XDG_TOPLEVEL_VERSION: c_int = 6;
const MODE_CLIENT_SIDE: u32 = 1;
const MODE_SERVER_SIDE: u32 = 2;
const EVENT_CONFIGURE: u32 = 0;
const ERROR_ALREADY_CONSTRUCTED: u32 = 1;
const ERROR_ORPHANED: u32 = 2;
const ERROR_INVALID_MODE: u32 = 3;
const MANAGER_INTERFACE_NAME: &CStr = c"zxdg_decoration_manager_v1";

#[repr(C)]
pub struct wl_display {
    _private: [u8; 0],
}

#[repr(C)]
pub struct wl_client {
    _private: [u8; 0],
}

#[repr(C)]
pub struct wl_global {
    _private: [u8; 0],
}

#[repr(C)]
pub struct wl_resource {
    _private: [u8; 0],
}

#[repr(C)]
pub struct wl_list {
    prev: *mut wl_list,
    next: *mut wl_list,
}

#[repr(C)]
pub struct wl_listener {
    link: wl_list,
    notify: Option<wl_notify_func_t>,
}

#[repr(C)]
pub struct wl_message {
    name: *const c_char,
    signature: *const c_char,
    types: *const *const wl_interface,
}

#[repr(C)]
pub struct wl_interface {
    name: *const c_char,
    version: c_int,
    method_count: c_int,
    methods: *const wl_message,
    event_count: c_int,
    events: *const wl_message,
}

unsafe impl Sync for wl_message {}
unsafe impl Sync for wl_interface {}

#[repr(transparent)]
struct InterfaceTypes<const N: usize>([*const wl_interface; N]);

unsafe impl<const N: usize> Sync for InterfaceTypes<N> {}

#[repr(transparent)]
struct ImplementationTable<const N: usize>([*const c_void; N]);

unsafe impl<const N: usize> Sync for ImplementationTable<N> {}

type wl_global_bind_func_t =
    unsafe extern "C" fn(client: *mut wl_client, data: *mut c_void, version: u32, id: u32);
type wl_notify_func_t = unsafe extern "C" fn(listener: *mut wl_listener, data: *mut c_void);
type wl_resource_destroy_func_t = unsafe extern "C" fn(resource: *mut wl_resource);
type wl_global_create_fn = unsafe extern "C" fn(
    display: *mut wl_display,
    interface: *const wl_interface,
    version: c_int,
    data: *mut c_void,
    bind: Option<wl_global_bind_func_t>,
) -> *mut wl_global;

#[link(name = "wayland-server")]
extern "C" {
    fn wl_client_post_no_memory(client: *mut wl_client);
    fn wl_resource_create(
        client: *mut wl_client,
        interface: *const wl_interface,
        version: c_int,
        id: u32,
    ) -> *mut wl_resource;
    fn wl_resource_destroy(resource: *mut wl_resource);
    fn wl_resource_get_version(resource: *mut wl_resource) -> c_int;
    fn wl_resource_get_user_data(resource: *mut wl_resource) -> *mut c_void;
    fn wl_resource_add_destroy_listener(resource: *mut wl_resource, listener: *mut wl_listener);
    fn wl_resource_post_event(resource: *mut wl_resource, opcode: u32, ...);
    fn wl_resource_post_error(resource: *mut wl_resource, code: u32, msg: *const c_char, ...);
    fn wl_resource_set_implementation(
        resource: *mut wl_resource,
        implementation: *const c_void,
        data: *mut c_void,
        destroy: Option<wl_resource_destroy_func_t>,
    );
    fn wl_list_remove(element: *mut wl_list);
}

static REAL_WL_GLOBAL_CREATE: OnceCell<wl_global_create_fn> = OnceCell::new();
static INJECTED_DISPLAYS: Lazy<Mutex<HashSet<usize>>> = Lazy::new(|| Mutex::new(HashSet::new()));
static DECORATED_TOPLEVELS: Lazy<Mutex<HashSet<usize>>> = Lazy::new(|| Mutex::new(HashSet::new()));

thread_local! {
    static IN_HAZE_GLOBAL_CREATE: Cell<bool> = const { Cell::new(false) };
}

static EMPTY_TYPES: InterfaceTypes<1> = InterfaceTypes([ptr::null()]);
static DECORATION_TYPES: InterfaceTypes<3> = InterfaceTypes([
    &TOPLEVEL_DECORATION_INTERFACE,
    &XDG_TOPLEVEL_INTERFACE,
    ptr::null(),
]);

static MANAGER_REQUESTS: [wl_message; 2] = [
    wl_message {
        name: c"destroy".as_ptr(),
        signature: c"".as_ptr(),
        types: EMPTY_TYPES.0.as_ptr(),
    },
    wl_message {
        name: c"get_toplevel_decoration".as_ptr(),
        signature: c"no".as_ptr(),
        types: DECORATION_TYPES.0.as_ptr(),
    },
];

static DECORATION_REQUESTS: [wl_message; 3] = [
    wl_message {
        name: c"destroy".as_ptr(),
        signature: c"".as_ptr(),
        types: EMPTY_TYPES.0.as_ptr(),
    },
    wl_message {
        name: c"set_mode".as_ptr(),
        signature: c"u".as_ptr(),
        types: EMPTY_TYPES.0.as_ptr(),
    },
    wl_message {
        name: c"unset_mode".as_ptr(),
        signature: c"".as_ptr(),
        types: EMPTY_TYPES.0.as_ptr(),
    },
];

static DECORATION_EVENTS: [wl_message; 1] = [wl_message {
    name: c"configure".as_ptr(),
    signature: c"u".as_ptr(),
    types: EMPTY_TYPES.0.as_ptr(),
}];

static MANAGER_INTERFACE: wl_interface = wl_interface {
    name: c"zxdg_decoration_manager_v1".as_ptr(),
    version: MANAGER_VERSION,
    method_count: MANAGER_REQUESTS.len() as c_int,
    methods: MANAGER_REQUESTS.as_ptr(),
    event_count: 0,
    events: ptr::null(),
};

static TOPLEVEL_DECORATION_INTERFACE: wl_interface = wl_interface {
    name: c"zxdg_toplevel_decoration_v1".as_ptr(),
    version: MANAGER_VERSION,
    method_count: DECORATION_REQUESTS.len() as c_int,
    methods: DECORATION_REQUESTS.as_ptr(),
    event_count: DECORATION_EVENTS.len() as c_int,
    events: DECORATION_EVENTS.as_ptr(),
};

static XDG_TOPLEVEL_INTERFACE: wl_interface = wl_interface {
    name: c"xdg_toplevel".as_ptr(),
    version: XDG_TOPLEVEL_VERSION,
    method_count: 0,
    methods: ptr::null(),
    event_count: 0,
    events: ptr::null(),
};

static MANAGER_IMPLEMENTATION: ImplementationTable<2> = ImplementationTable([
    manager_destroy as *const c_void,
    manager_get_toplevel_decoration as *const c_void,
]);
static DECORATION_IMPLEMENTATION: ImplementationTable<3> = ImplementationTable([
    decoration_destroy as *const c_void,
    decoration_set_mode as *const c_void,
    decoration_unset_mode as *const c_void,
]);

#[repr(C)]
struct DecorationState {
    toplevel_destroy_listener: wl_listener,
    toplevel: usize,
    decoration: *mut wl_resource,
    toplevel_listener_linked: bool,
}

fn real_wl_global_create() -> Option<wl_global_create_fn> {
    REAL_WL_GLOBAL_CREATE
        .get_or_try_init(|| unsafe {
            let symbol = libc::dlsym(libc::RTLD_NEXT, c"wl_global_create".as_ptr());
            if symbol.is_null() {
                Err(())
            } else {
                Ok(std::mem::transmute::<*mut c_void, wl_global_create_fn>(
                    symbol,
                ))
            }
        })
        .copied()
        .ok()
}

fn decorated_toplevels() -> Option<MutexGuard<'static, HashSet<usize>>> {
    DECORATED_TOPLEVELS.lock().ok()
}

unsafe fn is_decoration_manager_interface(interface: *const wl_interface) -> bool {
    if interface.is_null() || (*interface).name.is_null() {
        return false;
    }

    CStr::from_ptr((*interface).name) == MANAGER_INTERFACE_NAME
}

unsafe fn create_global_passthrough(
    display: *mut wl_display,
    interface: *const wl_interface,
    version: c_int,
    data: *mut c_void,
    bind: Option<wl_global_bind_func_t>,
) -> *mut wl_global {
    match real_wl_global_create() {
        Some(real) => real(display, interface, version, data, bind),
        None => ptr::null_mut(),
    }
}

unsafe fn maybe_advertise_decoration_manager(display: *mut wl_display) {
    if display.is_null() {
        return;
    }

    let should_create = {
        let mut displays = match INJECTED_DISPLAYS.lock() {
            Ok(displays) => displays,
            Err(_) => return,
        };
        displays.insert(display as usize)
    };
    if !should_create {
        return;
    }

    IN_HAZE_GLOBAL_CREATE.with(|guard| {
        if guard.get() {
            return;
        }
        guard.set(true);
        let _global = create_global_passthrough(
            display,
            &MANAGER_INTERFACE,
            MANAGER_VERSION,
            ptr::null_mut(),
            Some(bind_decoration_manager),
        );
        guard.set(false);
    });
}

unsafe extern "C" fn bind_decoration_manager(
    client: *mut wl_client,
    _data: *mut c_void,
    version: u32,
    id: u32,
) {
    if client.is_null() {
        return;
    }

    let version = version.min(MANAGER_VERSION as u32) as c_int;
    let resource = wl_resource_create(client, &MANAGER_INTERFACE, version, id);
    if resource.is_null() {
        wl_client_post_no_memory(client);
        return;
    }

    wl_resource_set_implementation(
        resource,
        MANAGER_IMPLEMENTATION.0.as_ptr() as *const c_void,
        ptr::null_mut(),
        None,
    );
}

unsafe extern "C" fn manager_destroy(_client: *mut wl_client, resource: *mut wl_resource) {
    if !resource.is_null() {
        wl_resource_destroy(resource);
    }
}

unsafe extern "C" fn manager_get_toplevel_decoration(
    client: *mut wl_client,
    resource: *mut wl_resource,
    id: u32,
    toplevel: *mut wl_resource,
) {
    if client.is_null() || resource.is_null() {
        return;
    }
    if toplevel.is_null() {
        wl_resource_post_error(resource, ERROR_ORPHANED, c"xdg_toplevel is null".as_ptr());
        return;
    }

    let toplevel_key = toplevel as usize;
    let mut decorated = match decorated_toplevels() {
        Some(decorated) => decorated,
        None => return,
    };
    if !decorated.insert(toplevel_key) {
        wl_resource_post_error(
            resource,
            ERROR_ALREADY_CONSTRUCTED,
            c"xdg_toplevel already has a decoration object".as_ptr(),
        );
        return;
    }

    let version = wl_resource_get_version(resource).clamp(1, MANAGER_VERSION);
    let decoration = wl_resource_create(client, &TOPLEVEL_DECORATION_INTERFACE, version, id);
    if decoration.is_null() {
        decorated.remove(&toplevel_key);
        wl_client_post_no_memory(client);
        return;
    }
    drop(decorated);

    let state = Box::new(DecorationState {
        toplevel_destroy_listener: wl_listener {
            link: wl_list {
                prev: ptr::null_mut(),
                next: ptr::null_mut(),
            },
            notify: Some(toplevel_destroyed_before_decoration),
        },
        toplevel: toplevel_key,
        decoration,
        toplevel_listener_linked: true,
    });
    let state = Box::into_raw(state);
    wl_resource_add_destroy_listener(toplevel, &mut (*state).toplevel_destroy_listener);

    wl_resource_set_implementation(
        decoration,
        DECORATION_IMPLEMENTATION.0.as_ptr() as *const c_void,
        state as *mut c_void,
        Some(decoration_state_destroy),
    );
    wl_resource_post_event(decoration, EVENT_CONFIGURE, MODE_CLIENT_SIDE);
}

unsafe extern "C" fn decoration_destroy(_client: *mut wl_client, resource: *mut wl_resource) {
    if !resource.is_null() {
        wl_resource_destroy(resource);
    }
}

unsafe extern "C" fn decoration_state_destroy(resource: *mut wl_resource) {
    if resource.is_null() {
        return;
    }

    let state = wl_resource_get_user_data(resource) as *mut DecorationState;
    if state.is_null() {
        return;
    }

    let state = Box::from_raw(state);
    if state.toplevel_listener_linked {
        wl_list_remove(&state.toplevel_destroy_listener.link as *const wl_list as *mut wl_list);
    }
    if let Some(mut decorated) = decorated_toplevels() {
        decorated.remove(&state.toplevel);
    }
}

unsafe extern "C" fn toplevel_destroyed_before_decoration(
    listener: *mut wl_listener,
    _data: *mut c_void,
) {
    if listener.is_null() {
        return;
    }

    let state = listener as *mut DecorationState;
    (*state).toplevel_listener_linked = false;
    if let Some(mut decorated) = decorated_toplevels() {
        decorated.remove(&(*state).toplevel);
    }
    if !(*state).decoration.is_null() {
        wl_resource_post_error(
            (*state).decoration,
            ERROR_ORPHANED,
            c"xdg_toplevel destroyed before its decoration object".as_ptr(),
        );
    }
}

unsafe extern "C" fn decoration_set_mode(
    _client: *mut wl_client,
    resource: *mut wl_resource,
    mode: u32,
) {
    if !resource.is_null() {
        match mode {
            MODE_CLIENT_SIDE => wl_resource_post_event(resource, EVENT_CONFIGURE, MODE_CLIENT_SIDE),
            MODE_SERVER_SIDE => wl_resource_post_event(resource, EVENT_CONFIGURE, MODE_SERVER_SIDE),
            _ => wl_resource_post_error(
                resource,
                ERROR_INVALID_MODE,
                c"invalid zxdg_toplevel_decoration_v1 mode".as_ptr(),
            ),
        }
    }
}

unsafe extern "C" fn decoration_unset_mode(_client: *mut wl_client, resource: *mut wl_resource) {
    if !resource.is_null() {
        wl_resource_post_event(resource, EVENT_CONFIGURE, MODE_SERVER_SIDE);
    }
}

#[no_mangle]
/// Interposes Mutter's `wl_global_create` calls and injects Haze's decoration
/// manager global after forwarding the original global creation.
///
/// # Safety
///
/// This function is called by `libwayland-server` through the C ABI. The caller
/// must pass the same pointer arguments and callback contract required by the
/// real `wl_global_create`; Haze forwards them unchanged before doing any
/// optional decoration-manager injection.
pub unsafe extern "C" fn wl_global_create(
    display: *mut wl_display,
    interface: *const wl_interface,
    version: c_int,
    data: *mut c_void,
    bind: Option<wl_global_bind_func_t>,
) -> *mut wl_global {
    let compositor_provides_decoration_manager = is_decoration_manager_interface(interface);
    let global = create_global_passthrough(display, interface, version, data, bind);

    let in_haze = IN_HAZE_GLOBAL_CREATE.with(Cell::get);
    if !in_haze && !global.is_null() && !compositor_provides_decoration_manager {
        maybe_advertise_decoration_manager(display);
    }

    global
}

#[no_mangle]
pub extern "C" fn haze_wayland_ssd_supported() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn haze_wayland_ssd_version() -> *const c_char {
    VERSION.as_ptr()
}

#[no_mangle]
pub extern "C" fn haze_wayland_ssd_host_status() -> *const c_char {
    HOST_STATUS.as_ptr()
}

mod haze_window_frame {
    use super::*;

    #[derive(Default)]
    pub struct HazeWindowFrame {
        pub is_focused: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HazeWindowFrame {
        const NAME: &'static str = "HazeWindowFrame";
        type Type = super::HazeWindowFrame;
    }

    impl ObjectImpl for HazeWindowFrame {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoolean::builder("is-focused")
                    .nick("Is focused")
                    .blurb("Whether the unmanaged Haze frame tracks an active toplevel")
                    .default_value(false)
                    .readwrite()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, id: usize, value: &glib::Value, _pspec: &glib::ParamSpec) {
            if id == 1 {
                if let Ok(is_focused) = value.get::<bool>() {
                    self.is_focused.set(is_focused);
                }
            }
        }

        fn property(&self, id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            match id {
                1 => self.is_focused.get().to_value(),
                _ => false.to_value(),
            }
        }
    }
}

glib::wrapper! {
    pub struct HazeWindowFrame(ObjectSubclass<haze_window_frame::HazeWindowFrame>);
}

#[no_mangle]
pub extern "C" fn haze_wayland_ssd_register_types() {
    let _ = HazeWindowFrame::static_type();
}

#[no_mangle]
pub extern "C" fn haze_wayland_ssd_new_window_frame() -> *mut glib::gobject_ffi::GObject {
    let frame: HazeWindowFrame = glib::Object::new();
    frame.upcast::<glib::Object>().into_glib_ptr()
}
