#![allow(improper_ctypes_definitions)]
#![allow(non_snake_case)]
use jni::Env;
use jni::objects::{JClass, JString, JValue};
use jni::strings::JNIString;
use jni::sys::{jlong, jobject, jobjectArray, jstring};

use crate::aam::AAM;
use crate::error::AamlError;

fn throw_java_exception(env: &mut Env<'_>, class: &str, msg: impl ToString) {
    let _ = env.throw_new(JNIString::from(class), JNIString::from(msg.to_string()));
}

fn java_string_to_rust(env: &mut Env<'_>, value: &JString<'_>) -> Result<String, String> {
    value.try_to_string(env).map_err(|e| e.to_string())
}

fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,

        content: String::new(),

        details: "unexpected empty parse error list".to_string(),

        diagnostics: None,
    })
}

unsafe fn get_aam<'a>(ptr: jlong) -> Option<&'a AAM> {
    if ptr == 0 {
        return None;
    }

    Some(unsafe { &*(ptr as *const AAM) })
}

unsafe fn get_aam_mut<'a>(ptr: jlong) -> Option<&'a mut AAM> {
    if ptr == 0 {
        return None;
    }

    Some(unsafe { &mut *(ptr as *mut AAM) })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_new<'local>(
    mut _env: Env<'local>,
    _class: JClass<'local>,
) -> jlong {
    Box::into_raw(Box::new(AAM::new())) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_parse<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    content: JString<'local>,
) -> jlong {
    let content = match java_string_to_rust(&mut env, &content) {
        Ok(v) => v,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalArgumentException", e);
            return 0;
        }
    };

    match AAM::parse(&content) {
        Ok(aam) => Box::into_raw(Box::new(aam)) as jlong,
        Err(e) => {
            throw_java_exception(&mut env, "java/lang/IllegalStateException", first_error(e));
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_get<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    key: JString<'local>,
) -> jstring {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let key = match java_string_to_rust(&mut env, &key) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    if let Some(found) = aam.get(&key) {
        if let Ok(js) = env.new_string(found) {
            return js.into_raw();
        }
    }
    std::ptr::null_mut()
}

fn vec_to_jobject_array<'local>(env: &mut Env<'local>, list: Vec<&str>) -> jobjectArray {
    let class_string = env.find_class(jni::jni_str!("java/lang/String")).unwrap();
    let initial_str = env.new_string("").unwrap();
    let array = env
        .new_object_array(list.len() as i32, class_string, initial_str)
        .unwrap();

    for (i, item) in list.into_iter().enumerate() {
        let js = env.new_string(item).unwrap();
        array.set_element(env, i, js).unwrap();
    }
    array.into_raw()
}

fn map_to_java_hashmap<'local>(env: &mut Env<'local>, map: Vec<(&str, &str)>) -> jobject {
    let class_hashmap = env.find_class(jni::jni_str!("java/util/HashMap")).unwrap();
    let hashmap = env
        .new_object(&class_hashmap, jni::jni_sig!("()V"), &[])
        .unwrap();

    for (k, v) in map {
        let jk = env.new_string(k).unwrap();
        let jv = env.new_string(v).unwrap();
        env.call_method(
            &hashmap,
            jni::jni_str!("put"),
            jni::jni_sig!("(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;"),
            &[JValue::Object(&jk.into()), JValue::Object(&jv.into())],
        )
        .unwrap();
    }
    hashmap.into_raw()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_reverseSearch<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    value: JString<'local>,
) -> jobjectArray {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let value = java_string_to_rust(&mut env, &value).unwrap_or_default();

    let keys = aam.reverse_search(&value);
    vec_to_jobject_array(&mut env, keys)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_deepSearch<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
    pattern: JString<'local>,
) -> jobject {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };
    let pattern = java_string_to_rust(&mut env, &pattern).unwrap_or_default();

    let results = aam.deep_search(&pattern);
    map_to_java_hashmap(&mut env, results)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_schemaNames<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
) -> jobjectArray {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };

    if let Some(schemas) = aam.schemas() {
        let keys: Vec<&str> = schemas.keys().map(|k| k.as_str()).collect();
        vec_to_jobject_array(&mut env, keys)
    } else {
        std::ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustgames_aam_AAM_typeNames<'local>(
    mut env: Env<'local>,
    _class: JClass<'local>,
    ptr: jlong,
) -> jobjectArray {
    let Some(aam) = (unsafe { get_aam(ptr) }) else {
        return std::ptr::null_mut();
    };

    if let Some(types) = aam.types() {
        let keys: Vec<&str> = types.keys().map(|k| k.as_str()).collect();
        vec_to_jobject_array(&mut env, keys)
    } else {
        std::ptr::null_mut()
    }
}
