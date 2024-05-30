use crate::Plugin;
use crate::PluginType;
use crate::ColorModel;
use crate::ParamType;
use crate::Param;

pub use frei0r_sys::f0r_plugin_info_t;
pub use frei0r_sys::f0r_param_info_t;
pub use frei0r_sys::f0r_instance_t;
pub use frei0r_sys::f0r_param_t;

pub use std::ffi::c_int;
pub use std::ffi::c_uint;
pub use std::ffi::c_void;

use frei0r_sys::*;
use std::ffi::CStr;

pub unsafe extern "C" fn f0r_init() -> c_int { 1 }
pub unsafe extern "C" fn f0r_deinit() {}

pub unsafe extern "C" fn f0r_get_plugin_info<P: Plugin>(info: *mut f0r_plugin_info_t) {
    let info = unsafe { &mut *info };
    let our_info = P::info();

    info.name = our_info.name.as_ptr();
    info.author = our_info.author.as_ptr();
    info.plugin_type = match our_info.plugin_type {
        PluginType::Filter => F0R_PLUGIN_TYPE_FILTER as i32,
        PluginType::Source => F0R_PLUGIN_TYPE_SOURCE as i32,
        PluginType::Mixer2 => F0R_PLUGIN_TYPE_MIXER2 as i32,
        PluginType::Mixer3 => F0R_PLUGIN_TYPE_MIXER3 as i32,
    };
    info.color_model = match our_info.color_model {
        ColorModel::BGRA8888 => F0R_COLOR_MODEL_BGRA8888 as i32,
        ColorModel::RGBA8888 => F0R_COLOR_MODEL_RGBA8888 as i32,
        ColorModel::PACKED32 => F0R_COLOR_MODEL_PACKED32 as i32,
    };
    info.frei0r_version = FREI0R_MAJOR_VERSION as i32;
    info.major_version = our_info.major_version;
    info.minor_version = our_info.minor_version;
    info.num_params = our_info.num_params.try_into().unwrap();
    info.explanation = our_info.explanation.as_ptr();
}

pub unsafe fn f0r_get_param_info<P: Plugin>(info: *mut f0r_param_info_t, param_index: c_int) {
    let param_index = param_index.try_into().unwrap();

    let info = unsafe { &mut *info };
    let our_info = P::param_info(param_index);

    info.name = our_info.name.as_ptr();
    info.type_ = match our_info.param_type {
        ParamType::Bool     => F0R_PARAM_BOOL     as i32,
        ParamType::Double   => F0R_PARAM_DOUBLE   as i32,
        ParamType::Color    => F0R_PARAM_COLOR    as i32,
        ParamType::Position => F0R_PARAM_POSITION as i32,
        ParamType::String   => F0R_PARAM_STRING   as i32,
    };
    info.explanation = our_info.explanation.as_ptr();
}

pub struct Instance<P: Plugin> {
    width : usize,
    height : usize,
    inner : P,
}

pub unsafe extern "C" fn f0r_construct<P: Plugin>(width : c_uint, height: c_uint) -> f0r_instance_t {
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();
    let instance = P::new(width, height);
    let instance = Instance { width, height, inner: instance, };
    Box::into_raw(Box::new(instance)) as f0r_instance_t
}

pub unsafe extern "C" fn f0r_destruct<P: Plugin>(instance: f0r_instance_t) {
    let instance = unsafe { Box::from_raw(instance as *mut Instance<P>) };
    drop(instance)
}

pub unsafe extern "C" fn f0r_set_param_value<P: Plugin>(instance: f0r_instance_t, param: f0r_param_t, param_index: c_int) {
    let param_index = param_index.try_into().unwrap();

    let instance = unsafe { &mut *(instance as *mut Instance<P>) };
    match instance.inner.param_mut(param_index) {
        Param::Bool(value) => {
            assert!(P::param_info(param_index).param_type == ParamType::Bool);

            let param = unsafe { *(param as *const f0r_param_bool) };
            *value = param >= 0.5;
        },
        Param::Double(value) => {
            assert!(P::param_info(param_index).param_type == ParamType::Double);

            let param = unsafe { *(param as *const f0r_param_double) };
            *value = param;
        },
        Param::Color { r, g, b } => {
            assert!(P::param_info(param_index).param_type == ParamType::Color);

            let param = unsafe { *(param as *const f0r_param_color) };
            *r = param.r;
            *g = param.g;
            *b = param.b;
        },
        Param::Position { x, y } => {
            assert!(P::param_info(param_index).param_type == ParamType::Position);

            let param = unsafe { *(param as *const f0r_param_position) };
            *x = param.x;
            *y = param.y;
        },
        Param::String(value) => {
            assert!(P::param_info(param_index).param_type == ParamType::String);

            let param = unsafe { *(param as *const f0r_param_string) };
            *value = unsafe { CStr::from_ptr(param) }.to_owned();
        },
    };
}


pub unsafe extern "C" fn f0r_get_param_value<P: Plugin>(instance: f0r_instance_t, param: f0r_param_t, param_index: c_int) {
    let param_index = param_index.try_into().unwrap();

    let instance = unsafe { &mut *(instance as *mut Instance<P>) };
    match instance.inner.param(param_index) {
        Param::Bool(value) => {
            assert!(P::param_info(param_index).param_type == ParamType::Bool);

            let param = unsafe { &mut *(param as *mut f0r_param_bool) };
            *param = if *value { 1.0 } else { 0.0 };
        },
        Param::Double(value) => {
            assert!(P::param_info(param_index).param_type == ParamType::Double);

            let param = unsafe { &mut *(param as *mut f0r_param_double) };
            *param = *value;
        },
        Param::Color { r, g, b } => {
            assert!(P::param_info(param_index).param_type == ParamType::Color);

            let param = unsafe { &mut *(param as *mut f0r_param_color) };
            param.r = *r;
            param.g = *g;
            param.b = *b;
        },
        Param::Position { x, y } => {
            assert!(P::param_info(param_index).param_type == ParamType::Position);

            let param = unsafe { &mut *(param as *mut f0r_param_position) };
            param.x = *x;
            param.y = *y;
        },
        Param::String(value) => {
            assert!(P::param_info(param_index).param_type == ParamType::String);

            let param = unsafe { &mut *(param as *mut f0r_param_string) };
            *param = value.as_ptr() as f0r_param_string; // We are casting away constness here.
                                                         // This should be fine since quoting the
                                                         // comment found in the original header,
                                                         // "If the caller needs to modify the
                                                         // value, it should make a copy of it and
                                                         // modify before calling
                                                         // f0r_set_param_value()."
        },
    };
}

pub unsafe extern "C" fn f0r_update<P: Plugin>(instance: f0r_instance_t, time: f64, inframe: *const u32, outframe: *mut u32) {
    let instance = &mut *(instance as *mut Instance<P>);
    let inframe = std::slice::from_raw_parts(inframe, instance.width * instance.height);
    let outframe = std::slice::from_raw_parts_mut(outframe, instance.width * instance.height);
    instance.inner.update(time, instance.width, instance.height, inframe, outframe);
}

pub unsafe extern "C" fn f0r_update2<P: Plugin>(instance: f0r_instance_t, time: f64, inframe1: *const u32, inframe2: *const u32, inframe3: *const u32, outframe: *mut u32) {
    let instance = &mut *(instance as *mut Instance<P>);
    let inframe1 = std::slice::from_raw_parts(inframe1, instance.width * instance.height);
    let inframe2 = std::slice::from_raw_parts(inframe2, instance.width * instance.height);
    let inframe3 = std::slice::from_raw_parts(inframe3, instance.width * instance.height);
    let outframe = std::slice::from_raw_parts_mut(outframe, instance.width * instance.height);
    instance.inner.update2(time, instance.width, instance.height, inframe1, inframe2, inframe3, outframe);
}
