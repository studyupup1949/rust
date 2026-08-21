#![allow(improper_ctypes_definitions)]
#![allow(non_snake_case)]

use jni::Env;
use jni::objects::{JClass, JObject, JObjectArray, JString};
use jni::strings::JNIString;
use jni::sys::{jlong, jstring};

use crate::aam::AAM;
#[cfg(feature = "reconstructer")]
use crate::reconstructer;

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
        // SAFETY: the pointer is only ever produced by `Box::into_raw` and is
        // checked against null by callers that have ensured it is still valid.
        Some(unsafe { &*(ptr as *const AAM) })
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

#[cfg(feature = "reconstructer")]
fn aam_reconstruct_schema_impl(mut env: Env, name: JString, contents: JObjectArray) -> jstring {
    let name_str = match java_string_to_rust(&mut env, &name) {
        Ok(v) => v,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalArgumentException", e);
            return std::ptr::null_mut();
        }
    };

    let len = env.get_array_length(&contents).unwrap_or(0) as usize;
    let mut sources = Vec::with_capacity(len);

    for i in 0..len {
        let elem = env
            .get_object_array_element(&contents, i)
            .unwrap_or(JObject::null());
        if elem.is_null() {
            continue;
        }
        let js: JString = unsafe { JString::from_raw(&mut env, elem.into_raw()) };
        match java_string_to_rust(&mut env, &js) {
            Ok(s) => sources.push(s),
            Err(_) => continue,
        }
    }

    let refs: Vec<&str> = sources.iter().map(String::as_str).collect();
    match reconstructer::reconstruct_schema(&name_str, &refs) {
        Ok(formatted) => {
            if let Ok(js) = env.new_string(formatted) {
                return js.into_raw();
            }
            std::ptr::null_mut()
        }
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalStateException", e);
            std::ptr::null_mut()
        }
    }
}

// --- JNI Exports (New Package: rs.in.ininids.aam_jv) ---

#[unsafe(no_mangle)]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_new(_env: Env, _class: JClass) -> jlong {
    aam_new_impl()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_parse(
    env: Env,
    _class: JClass,
    content: JString,
) -> jlong {
    aam_parse_impl(env, content)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_get(
    mut env: Env,
    _class: JClass,
    ptr: jlong,
    key: JString,
) -> jstring {
    aam_get_impl(&mut env, ptr, key)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_delete(
    _env: Env,
    _class: JClass,
    ptr: jlong,
) {
    aam_delete_impl(ptr)
}

#[cfg(feature = "reconstructer")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_rs_in_ininids_aam_1jv_AAM_reconstructSchema(
    env: Env,
    _class: JClass,
    name: JString,
    contents: JObjectArray,
) -> jstring {
    aam_reconstruct_schema_impl(env, name, contents)
}

// --- JNI Exports (Legacy: com.rustgames.aam) ---

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_new(_env: Env, _class: JClass) -> jlong {
    aam_new_impl()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_parse(
    env: Env,
    _class: JClass,
    content: JString,
) -> jlong {
    aam_parse_impl(env, content)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_get(
    mut env: Env,
    _class: JClass,
    ptr: jlong,
    key: JString,
) -> jstring {
    aam_get_impl(&mut env, ptr, key)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_delete(_env: Env, _class: JClass, ptr: jlong) {
    aam_delete_impl(ptr)
}

#[cfg(feature = "reconstructer")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_reconstructSchema(
    env: Env,
    _class: JClass,
    name: JString,
    contents: JObjectArray,
) -> jstring {
    aam_reconstruct_schema_impl(env, name, contents)
}
