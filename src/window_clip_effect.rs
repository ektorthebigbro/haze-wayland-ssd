// SPDX-License-Identifier: GPL-3.0
// Copyright (C) 2026 Haze Project

use libc::{c_char, c_float, c_int, c_uint, c_ulong, c_void};
use once_cell::sync::OnceCell;
use std::mem;
use std::ptr;

type gboolean = c_int;
type guint = c_uint;
type guint16 = u16;
type gsize = usize;
type GType = c_ulong;
type GTypeFlags = c_uint;
type GParamFlags = c_uint;

const G_TYPE_FLAG_NONE: GTypeFlags = 0;
const G_PARAM_READWRITE: GParamFlags = 0x3;
const G_PARAM_EXPLICIT_NOTIFY: GParamFlags = 1 << 30;
const COGL_PIPELINE_FILTER_LINEAR: c_int = 1;
const COGL_PIPELINE_WRAP_MODE_CLAMP_TO_EDGE: c_int = 2;
const COGL_SNIPPET_HOOK_FRAGMENT: c_int = 2048;
const TYPE_NAME: &[u8] = b"HazeWindowClipEffect\0";

#[repr(C)]
struct GTypeClass {
    g_type: GType,
}

#[repr(C)]
struct GTypeInstance {
    g_class: *mut GTypeClass,
}

#[repr(C)]
struct GTypeQuery {
    type_: GType,
    type_name: *const c_char,
    class_size: guint,
    instance_size: guint,
}

#[repr(C)]
struct GTypeInfo {
    class_size: guint16,
    base_init: Option<unsafe extern "C" fn(*mut c_void)>,
    base_finalize: Option<unsafe extern "C" fn(*mut c_void)>,
    class_init: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    class_finalize: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    class_data: *const c_void,
    instance_size: guint16,
    n_preallocs: guint16,
    instance_init: Option<unsafe extern "C" fn(*mut GTypeInstance, *mut c_void)>,
    value_table: *const c_void,
}

#[repr(C)]
struct GObjectClass {
    g_type_class: GTypeClass,
    construct_properties: *mut c_void,
    constructor: *mut c_void,
    set_property: Option<unsafe extern "C" fn(*mut GObject, guint, *const GValue, *mut GParamSpec)>,
    get_property: Option<unsafe extern "C" fn(*mut GObject, guint, *mut GValue, *mut GParamSpec)>,
    dispose: Option<unsafe extern "C" fn(*mut GObject)>,
    finalize: *mut c_void,
    dispatch_properties_changed: *mut c_void,
    notify: *mut c_void,
    constructed: *mut c_void,
    flags: gsize,
    n_construct_properties: gsize,
    pspecs: *mut c_void,
    n_pspecs: gsize,
    pdummy: [*mut c_void; 3],
}

#[repr(C)]
struct ClutterActorMetaClass {
    parent_class: GObjectClass,
    set_actor: *mut c_void,
    set_enabled: *mut c_void,
}

#[repr(C)]
struct ClutterEffectClass {
    parent_class: ClutterActorMetaClass,
    pre_paint: *mut c_void,
    post_paint: *mut c_void,
    modify_paint_volume: *mut c_void,
    paint: *mut c_void,
    paint_node: *mut c_void,
    pick: *mut c_void,
}

#[repr(C)]
struct ClutterOffscreenEffectClass {
    parent_class: ClutterEffectClass,
    create_texture: *mut c_void,
    create_pipeline: Option<
        unsafe extern "C" fn(
            effect: *mut ClutterOffscreenEffect,
            texture: *mut CoglTexture,
        ) -> *mut CoglPipeline,
    >,
    paint_target: Option<
        unsafe extern "C" fn(
            effect: *mut ClutterOffscreenEffect,
            node: *mut ClutterPaintNode,
            paint_context: *mut ClutterPaintContext,
        ),
    >,
}

#[repr(C)]
struct GObject {
    _private: [u8; 0],
}
#[repr(C)]
struct GValue {
    _private: [u8; 0],
}
#[repr(C)]
struct GParamSpec {
    _private: [u8; 0],
}
#[repr(C)]
struct ClutterBackend {
    _private: [u8; 0],
}
#[repr(C)]
struct ClutterOffscreenEffect {
    _private: [u8; 0],
}
#[repr(C)]
struct ClutterPaintNode {
    _private: [u8; 0],
}
#[repr(C)]
struct ClutterPaintContext {
    _private: [u8; 0],
}
#[repr(C)]
struct CoglContext {
    _private: [u8; 0],
}
#[repr(C)]
struct CoglPipeline {
    _private: [u8; 0],
}
#[repr(C)]
struct CoglSnippet {
    _private: [u8; 0],
}
#[repr(C)]
struct CoglTexture {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Default)]
struct ClipPrivate {
    pipeline: *mut CoglPipeline,
    radii_uniform: c_int,
    size_uniform: c_int,
    top_left_radius: c_float,
    top_right_radius: c_float,
    bottom_right_radius: c_float,
    bottom_left_radius: c_float,
}

static EFFECT_TYPE: OnceCell<GType> = OnceCell::new();
static PARENT_PAINT_TARGET: OnceCell<
    unsafe extern "C" fn(
        *mut ClutterOffscreenEffect,
        *mut ClutterPaintNode,
        *mut ClutterPaintContext,
    ),
> = OnceCell::new();

unsafe extern "C" {
    fn g_type_query(type_: GType, query: *mut GTypeQuery);
    fn g_type_register_static(
        parent_type: GType,
        type_name: *const c_char,
        info: *const GTypeInfo,
        flags: GTypeFlags,
    ) -> GType;
    fn g_type_add_instance_private(class_type: GType, private_size: gsize) -> c_int;
    fn g_type_instance_get_private(
        instance: *mut GTypeInstance,
        private_type: GType,
    ) -> *mut c_void;
    fn g_type_class_peek_parent(g_class: *mut c_void) -> *mut c_void;

    fn g_param_spec_float(
        name: *const c_char,
        nick: *const c_char,
        blurb: *const c_char,
        minimum: c_float,
        maximum: c_float,
        default_value: c_float,
        flags: GParamFlags,
    ) -> *mut GParamSpec;
    fn g_object_class_install_property(
        oclass: *mut GObjectClass,
        property_id: guint,
        pspec: *mut GParamSpec,
    );
    fn g_object_notify_by_pspec(object: *mut GObject, pspec: *mut GParamSpec);
    fn g_value_get_float(value: *const GValue) -> c_float;
    fn g_value_set_float(value: *mut GValue, v_float: c_float);
    fn g_object_ref(object: *mut c_void) -> *mut c_void;
    fn g_object_unref(object: *mut c_void);

    fn clutter_offscreen_effect_get_type() -> GType;
    fn clutter_get_default_backend() -> *mut ClutterBackend;
    fn clutter_backend_get_cogl_context(backend: *mut ClutterBackend) -> *mut CoglContext;
    fn clutter_offscreen_effect_get_target_size(
        effect: *mut ClutterOffscreenEffect,
        width: *mut c_float,
        height: *mut c_float,
    ) -> gboolean;
    fn clutter_effect_queue_repaint(effect: *mut ClutterOffscreenEffect);

    fn cogl_pipeline_new(context: *mut CoglContext) -> *mut CoglPipeline;
    fn cogl_pipeline_set_layer_null_texture(pipeline: *mut CoglPipeline, layer_index: c_int);
    fn cogl_pipeline_set_layer_texture(
        pipeline: *mut CoglPipeline,
        layer_index: c_int,
        texture: *mut CoglTexture,
    );
    fn cogl_pipeline_set_layer_filters(
        pipeline: *mut CoglPipeline,
        layer_index: c_int,
        min_filter: c_int,
        mag_filter: c_int,
    );
    fn cogl_pipeline_set_layer_wrap_mode(
        pipeline: *mut CoglPipeline,
        layer_index: c_int,
        mode: c_int,
    );
    fn cogl_pipeline_add_snippet(pipeline: *mut CoglPipeline, snippet: *mut CoglSnippet);
    fn cogl_pipeline_get_uniform_location(
        pipeline: *mut CoglPipeline,
        uniform_name: *const c_char,
    ) -> c_int;
    fn cogl_pipeline_set_uniform_float(
        pipeline: *mut CoglPipeline,
        uniform_location: c_int,
        n_components: c_int,
        count: c_int,
        value: *const c_float,
    );
    fn cogl_snippet_new(
        hook: c_int,
        declarations: *const c_char,
        post: *const c_char,
    ) -> *mut CoglSnippet;
}

fn sanitize_radius(value: c_float) -> c_float {
    if value.is_finite() {
        value.clamp(0.0, 256.0)
    } else {
        0.0
    }
}

unsafe fn effect_private(object: *mut c_void) -> *mut ClipPrivate {
    let type_ = haze_window_clip_effect_get_type();
    g_type_instance_get_private(object.cast::<GTypeInstance>(), type_).cast::<ClipPrivate>()
}

unsafe fn create_pipeline() -> *mut CoglPipeline {
    let backend = clutter_get_default_backend();
    if backend.is_null() {
        return ptr::null_mut();
    }

    let context = clutter_backend_get_cogl_context(backend);
    if context.is_null() {
        return ptr::null_mut();
    }

    let pipeline = cogl_pipeline_new(context);
    if pipeline.is_null() {
        return ptr::null_mut();
    }

    cogl_pipeline_set_layer_null_texture(pipeline, 0);
    cogl_pipeline_set_layer_filters(
        pipeline,
        0,
        COGL_PIPELINE_FILTER_LINEAR,
        COGL_PIPELINE_FILTER_LINEAR,
    );
    cogl_pipeline_set_layer_wrap_mode(pipeline, 0, COGL_PIPELINE_WRAP_MODE_CLAMP_TO_EDGE);

    let snippet = cogl_snippet_new(
        COGL_SNIPPET_HOOK_FRAGMENT,
        c"uniform vec4 haze_corner_radii;\nuniform vec2 haze_actor_size;\n".as_ptr(),
        concat!(
            "  vec2 size = max(haze_actor_size, vec2(1.0));\n",
            "  vec2 p = cogl_tex_coord_in[0].st * size;\n",
            "  vec4 radii = clamp(haze_corner_radii, vec4(0.0), vec4(min(size.x, size.y) * 0.5));\n",
            "  float radius = 0.0;\n",
            "  vec2 center = p;\n",
            "  if (p.x < radii.x && p.y < radii.x) { radius = radii.x; center = vec2(radii.x, radii.x); }\n",
            "  else if (p.x > size.x - radii.y && p.y < radii.y) { radius = radii.y; center = vec2(size.x - radii.y, radii.y); }\n",
            "  else if (p.x > size.x - radii.z && p.y > size.y - radii.z) { radius = radii.z; center = vec2(size.x - radii.z, size.y - radii.z); }\n",
            "  else if (p.x < radii.w && p.y > size.y - radii.w) { radius = radii.w; center = vec2(radii.w, size.y - radii.w); }\n",
            "  if (radius > 0.0) { float dist = length(p - center); float mask = 1.0 - smoothstep(radius - 1.0, radius, dist); cogl_color_out.rgb *= mask; cogl_color_out.a *= mask; }\n",
            "\0"
        )
        .as_ptr()
        .cast(),
    );
    if !snippet.is_null() {
        cogl_pipeline_add_snippet(pipeline, snippet);
        g_object_unref(snippet.cast());
    }

    pipeline
}

unsafe extern "C" fn class_init(class: *mut c_void, _data: *mut c_void) {
    let object_class = class.cast::<GObjectClass>();
    (*object_class).set_property = Some(set_property);
    (*object_class).get_property = Some(get_property);
    (*object_class).dispose = Some(dispose);

    let offscreen_class = class.cast::<ClutterOffscreenEffectClass>();
    let parent = g_type_class_peek_parent(class).cast::<ClutterOffscreenEffectClass>();
    if !parent.is_null() {
        if let Some(parent_paint_target) = (*parent).paint_target {
            let _ = PARENT_PAINT_TARGET.set(parent_paint_target);
        }
    }
    (*offscreen_class).create_pipeline = Some(create_pipeline_vfunc);
    (*offscreen_class).paint_target = Some(paint_target_vfunc);

    let type_ = (*class.cast::<GTypeClass>()).g_type;
    g_type_add_instance_private(type_, mem::size_of::<ClipPrivate>());

    for (id, name, nick, blurb) in [
        (
            1,
            c"top-left-radius".as_ptr(),
            c"Top Left Radius".as_ptr(),
            c"Clip radius for the top-left app-content corner".as_ptr(),
        ),
        (
            2,
            c"top-right-radius".as_ptr(),
            c"Top Right Radius".as_ptr(),
            c"Clip radius for the top-right app-content corner".as_ptr(),
        ),
        (
            3,
            c"bottom-right-radius".as_ptr(),
            c"Bottom Right Radius".as_ptr(),
            c"Clip radius for the bottom-right app-content corner".as_ptr(),
        ),
        (
            4,
            c"bottom-left-radius".as_ptr(),
            c"Bottom Left Radius".as_ptr(),
            c"Clip radius for the bottom-left app-content corner".as_ptr(),
        ),
        (
            5,
            c"radius".as_ptr(),
            c"Radius".as_ptr(),
            c"Uniform app-content clip radius".as_ptr(),
        ),
    ] {
        let pspec = g_param_spec_float(
            name,
            nick,
            blurb,
            0.0,
            256.0,
            0.0,
            G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY,
        );
        g_object_class_install_property(object_class, id, pspec);
    }
}

unsafe extern "C" fn instance_init(instance: *mut GTypeInstance, _class: *mut c_void) {
    let private = effect_private(instance.cast());
    if private.is_null() {
        return;
    }
    ptr::write(
        private,
        ClipPrivate {
            radii_uniform: -1,
            size_uniform: -1,
            ..ClipPrivate::default()
        },
    );
    let pipeline = create_pipeline();
    (*private).pipeline = pipeline;
    if !pipeline.is_null() {
        (*private).radii_uniform =
            cogl_pipeline_get_uniform_location(pipeline, c"haze_corner_radii".as_ptr());
        (*private).size_uniform =
            cogl_pipeline_get_uniform_location(pipeline, c"haze_actor_size".as_ptr());
    }
}

unsafe extern "C" fn create_pipeline_vfunc(
    effect: *mut ClutterOffscreenEffect,
    texture: *mut CoglTexture,
) -> *mut CoglPipeline {
    let private = effect_private(effect.cast());
    if private.is_null() {
        return ptr::null_mut();
    }
    if (*private).pipeline.is_null() {
        (*private).pipeline = create_pipeline();
    }
    if (*private).pipeline.is_null() {
        return ptr::null_mut();
    }
    cogl_pipeline_set_layer_texture((*private).pipeline, 0, texture);
    g_object_ref((*private).pipeline.cast()).cast()
}

unsafe extern "C" fn paint_target_vfunc(
    effect: *mut ClutterOffscreenEffect,
    node: *mut ClutterPaintNode,
    paint_context: *mut ClutterPaintContext,
) {
    let private = effect_private(effect.cast());
    if !private.is_null() && !(*private).pipeline.is_null() {
        let mut width = 1.0;
        let mut height = 1.0;
        if clutter_offscreen_effect_get_target_size(effect, &mut width, &mut height) == 0 {
            width = 1.0;
            height = 1.0;
        }
        let size = [width.max(1.0), height.max(1.0)];
        let radii = [
            (*private).top_left_radius,
            (*private).top_right_radius,
            (*private).bottom_right_radius,
            (*private).bottom_left_radius,
        ];

        if (*private).radii_uniform >= 0 {
            cogl_pipeline_set_uniform_float(
                (*private).pipeline,
                (*private).radii_uniform,
                4,
                1,
                radii.as_ptr(),
            );
        }
        if (*private).size_uniform >= 0 {
            cogl_pipeline_set_uniform_float(
                (*private).pipeline,
                (*private).size_uniform,
                2,
                1,
                size.as_ptr(),
            );
        }
    }

    if let Some(parent_paint_target) = PARENT_PAINT_TARGET.get().copied() {
        parent_paint_target(effect, node, paint_context);
    }
}

unsafe extern "C" fn set_property(
    object: *mut GObject,
    property_id: guint,
    value: *const GValue,
    pspec: *mut GParamSpec,
) {
    let private = effect_private(object.cast());
    if private.is_null() {
        return;
    }

    let next = sanitize_radius(g_value_get_float(value));
    let changed = match property_id {
        1 => replace_if_changed(&mut (*private).top_left_radius, next),
        2 => replace_if_changed(&mut (*private).top_right_radius, next),
        3 => replace_if_changed(&mut (*private).bottom_right_radius, next),
        4 => replace_if_changed(&mut (*private).bottom_left_radius, next),
        5 => {
            let mut changed = false;
            changed |= replace_if_changed(&mut (*private).top_left_radius, next);
            changed |= replace_if_changed(&mut (*private).top_right_radius, next);
            changed |= replace_if_changed(&mut (*private).bottom_right_radius, next);
            changed |= replace_if_changed(&mut (*private).bottom_left_radius, next);
            changed
        }
        _ => false,
    };

    if changed {
        g_object_notify_by_pspec(object, pspec);
        clutter_effect_queue_repaint(object.cast());
    }
}

unsafe fn replace_if_changed(slot: &mut c_float, next: c_float) -> bool {
    if (*slot - next).abs() < c_float::EPSILON {
        false
    } else {
        *slot = next;
        true
    }
}

unsafe extern "C" fn get_property(
    object: *mut GObject,
    property_id: guint,
    value: *mut GValue,
    _pspec: *mut GParamSpec,
) {
    let private = effect_private(object.cast());
    if private.is_null() {
        g_value_set_float(value, 0.0);
        return;
    }

    let result = match property_id {
        1 => (*private).top_left_radius,
        2 => (*private).top_right_radius,
        3 => (*private).bottom_right_radius,
        4 => (*private).bottom_left_radius,
        5 => (*private).top_left_radius,
        _ => 0.0,
    };
    g_value_set_float(value, result);
}

unsafe extern "C" fn dispose(object: *mut GObject) {
    let private = effect_private(object.cast());
    if !private.is_null() && !(*private).pipeline.is_null() {
        g_object_unref((*private).pipeline.cast());
        (*private).pipeline = ptr::null_mut();
    }
}

pub fn haze_window_clip_effect_get_type() -> GType {
    *EFFECT_TYPE.get_or_init(|| unsafe {
        let parent_type = clutter_offscreen_effect_get_type();
        let mut query = GTypeQuery {
            type_: 0,
            type_name: ptr::null(),
            class_size: 0,
            instance_size: 0,
        };
        g_type_query(parent_type, &mut query);
        let info = GTypeInfo {
            class_size: query.class_size as guint16,
            base_init: None,
            base_finalize: None,
            class_init: Some(class_init),
            class_finalize: None,
            class_data: ptr::null(),
            instance_size: query.instance_size as guint16,
            n_preallocs: 0,
            instance_init: Some(instance_init),
            value_table: ptr::null(),
        };
        g_type_register_static(
            parent_type,
            TYPE_NAME.as_ptr().cast(),
            &info,
            G_TYPE_FLAG_NONE,
        )
    })
}

pub fn register_type() {
    let _ = haze_window_clip_effect_get_type();
}

pub fn is_supported() -> bool {
    haze_window_clip_effect_get_type() != 0
}
