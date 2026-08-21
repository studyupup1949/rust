#![allow(improper_ctypes_definitions)]
#![allow(non_snake_case)]

use jni::Env;
use jni::objects::{JClass, JString, JValue};
use jni::strings::JNIString;
use jni::sys::{jlong, jobject, jobjectArray, jstring};

use crate::aam::AAM;
use crate::error::AamlError;

// --- Helpers ---

fn throw_java_exception(env: &mut Env<'_>, class: &str, msg: impl ToString) {
    let _ = env.throw_new(JNIString::from(class), JNIString::from(msg.to_string()));
}

fn java_string_to_rust(env: &mut Env<'_>, value: &JString<'_>) -> Result<String, String> {
    value.try_to_string(env).map_err(|e| e.to_string())
}

unsafe fn get_aam<'a>(ptr: jlong) -> Option<&'a AAM> {
    if ptr == 0 {
        None
    } else {
        Some(&*(ptr as *const AAM))
    }
}

// --- Internal Logic ---

fn aam_new_impl() -> jlong {
    Box::into_raw(Box::new(AAM::new())) as jlong
}

fn aam_parse_impl(mut env: Env, content: JString) -> jlong {
    let content_str = match java_string_to_rust(&mut env, &content) {
        Ok(v) => v,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalArgumentException", e);
            return 0;
        }
    };

    match AAM::parse(&content_str) {
        Ok(aam) => Box::into_raw(Box::new(aam)) as jlong,
        Err(e) => {
            let msg = e
                .into_iter()
                .next()
                .map(|err| err.to_string())
                .unwrap_or_default();
            throw_java_exception(&mut env, "java/lang/IllegalStateException", msg);
            0
        }
    }
}

fn aam_get_impl(env: &mut Env, ptr: jlong, key: JString) -> jstring {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let key_str = java_string_to_rust(env, &key).unwrap_or_default();

    if let Some(found) = aam.get(&key_str) {
        if let Ok(js) = env.new_string(found) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

fn aam_delete_impl(ptr: jlong) {
    if ptr != 0 {
        unsafe {
            let _ = Box::from_raw(ptr as *mut AAM);
        }
    }
}

// --- JNI Exports (New Package: rs.in.ininids.aam_jv) ---

#[no_mangle]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_new(_env: Env, _class: JClass) -> jlong {
    aam_new_impl()
}

#[no_mangle]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_parse(
    env: Env,
    _class: JClass,
    content: JString,
) -> jlong {
    aam_parse_impl(env, content)
}

#[no_mangle]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_get(
    mut env: Env,
    _class: JClass,
    ptr: jlong,
    key: JString,
) -> jstring {
    aam_get_impl(&mut env, ptr, key)
}

#[no_mangle]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_delete(
    _env: Env,
    _class: JClass,
    ptr: jlong,
) {
    aam_delete_impl(ptr)
}

// --- JNI Exports (Legacy: com.rustgames.aam) ---

#[no_mangle]
pub extern "system" fn Java_com_rustgames_aam_AAM_new(_env: Env, _class: JClass) -> jlong {
    aam_new_impl()
}

#[no_mangle]
pub extern "system" fn Java_com_rustgames_aam_AAM_parse(
    env: Env,
    _class: JClass,
    content: JString,
) -> jlong {
    aam_parse_impl(env, content)
}

#[no_mangle]
pub extern "system" fn Java_com_rustgames_aam_AAM_get(
    mut env: Env,
    _class: JClass,
    ptr: jlong,
    key: JString,
) -> jstring {
    aam_get_impl(&mut env, ptr, key)
}

#[no_mangle]
pub extern "system" fn Java_com_rustgames_aam_AAM_delete(_env: Env, _class: JClass, ptr: jlong) {
    aam_delete_impl(ptr)
}
