use libc::c_char;
use serde_json;
use std::ffi::CStr;

mod api;
mod events;
mod settings;

use api::APIClient;
use events::{Event, EventLog};
use settings::Settings;

// http://jakegoulding.com/rust-ffi-omnibus/string_arguments/
// https://docs.rs/ffi-support/0.4.4/src/ffi_support/ffistr.rs.html#144
fn parse_ffi_str(s: *const c_char) -> String {
    let c_str = unsafe {
        assert!(!s.is_null());

        CStr::from_ptr(s)
    };
    c_str.to_string_lossy().to_string()
}

fn parse_ffi_json(s: *const c_char) -> serde_json::Value {
    let s = parse_ffi_str(s);
    serde_json::from_str(&s).expect("Error parsing JSON in CLS ffi library")
}

// Bools will be passed in as 0/1
// https://mozilla.github.io/application-services/book/howtos/when-to-use-what-in-the-ffi.html#primitives
fn parse_ffi_bool(i: u32) -> bool {
    if i == 0 {
        false
    } else {
        true
    }
}

use once_cell::sync::Lazy;

// Efectively the "global" settings variable
static mut SETTINGS: Lazy<Settings> = Lazy::new(|| Settings::new());

pub fn debug_print(s: String) {
    if unsafe { SETTINGS.get_debug() } {
        println!("CLS: {}", s);
    }
}

#[no_mangle]
pub extern "C" fn track_event(
    slug: *const c_char,
    type_s: *const c_char,
    metadata: *const c_char,
    dispatch: u32,
) {
    if slug.is_null() || type_s.is_null() || metadata.is_null() {
        return;
    }

    let slug = parse_ffi_str(slug);
    let type_s = parse_ffi_str(type_s);
    let metadata = parse_ffi_json(metadata);
    let dispatch = parse_ffi_bool(dispatch);

    debug_print(format!(
        "track_event slug={:?} type={:?} metadata={:?} dispatch={:?}",
        slug, type_s, metadata, dispatch
    ));

    let invocation_id = unsafe { SETTINGS.get_invocation_id() };
    let user_id = unsafe { SETTINGS.get_user_id() };
    let version = unsafe { SETTINGS.version.as_str() };
    let ci = unsafe { SETTINGS.get_is_ci() };
    let event = Event::new(
        &slug,
        &type_s,
        metadata,
        &user_id,
        &invocation_id,
        &ci,
        &version,
    );

    let should_track = match unsafe { SETTINGS.should_track_event(&event) } {
        Ok(val) => val,
        Err(_) => false,
    };
    if !should_track {
        return;
    }

    let mut should_record = !dispatch;

    if dispatch {
        let token = unsafe { SETTINGS.get_project_key() };
        let api = APIClient::new(&token);
        if api.post_event(&event).is_err() {
            should_record = true
        }
    }

    if should_record {
        let log = EventLog::new(unsafe { &SETTINGS.get_cache_dir() });
        log.record_event(&event);
    }
}

#[no_mangle]
pub extern "C" fn dispatch_events() {
    debug_print("dispatch_events".to_string());
    let log = EventLog::new(unsafe { &SETTINGS.get_cache_dir() });
    let events = log.get_events();
    let token = unsafe { SETTINGS.get_project_key() };
    let api = APIClient::new(&token);

    let mut events_failed = Vec::new();
    let mut events_succeded = Vec::new();

    for event in events {
        match api.post_event(&event) {
            Ok(_) => events_succeded.push(event),
            Err(_) => events_failed.push(event),
        }
    }
    if events_failed.len() > 0 {
        debug_print(format!(
            "{:?} events failed to dispatch",
            events_failed.len()
        ));
    }
    if events_succeded.len() > 0 {
        log.clear();
    }
}

#[no_mangle]
pub extern "C" fn set_project_key(key: *const c_char) {
    if key.is_null() {
        // Silently return
        return;
    }

    let key = parse_ffi_str(key);

    unsafe {
        SETTINGS.set_project_key(key.as_str());
        debug_print(format!(
            "set_project_key key={:?}",
            SETTINGS.get_project_key()
        ));
    }
}

#[no_mangle]
pub extern "C" fn set_version(key: *const c_char) {
    if key.is_null() {
        // Silently return
        return;
    }

    let key = parse_ffi_str(key);

    unsafe {
        SETTINGS.version = key;
        debug_print(format!("set_version key={:?}", SETTINGS.version));
    }
}

#[no_mangle]
pub extern "C" fn set_project_slug(slug: *const c_char) {
    if slug.is_null() {
        // Silently return
        return;
    }

    let slug = parse_ffi_str(slug);
    unsafe {
        SETTINGS.project_slug = slug;
        debug_print(format!("set_project_slug slug={:?}", SETTINGS.project_slug))
    }
}

#[no_mangle]
pub extern "C" fn set_instance_id(id: *const c_char) {
    if id.is_null() {
        // Silently return
        return;
    }

    let id = parse_ffi_str(id);
    unsafe {
        SETTINGS.instance_id = id;
        debug_print(format!("set_instance_id id={:?}", SETTINGS.instance_id))
    }
}

#[no_mangle]
pub extern "C" fn set_request_permission_prompt(text: *const c_char) {
    if text.is_null() {
        // Silently return
        return;
    }

    let text = parse_ffi_str(text);
    unsafe {
        SETTINGS.request_permission_prompt = text;
        debug_print(format!(
            "set_request_permission_prompt text={:?}",
            SETTINGS.request_permission_prompt
        ))
    }
}

#[no_mangle]
pub extern "C" fn set_debug(debug: u32) {
    let debug = parse_ffi_bool(debug);
    unsafe {
        SETTINGS.set_debug(debug);
        debug_print(format!("set_debug debug={:?}", SETTINGS.get_debug()))
    }
}

#[no_mangle]
pub extern "C" fn set_is_ci(is_ci: u32) {
    let is_ci = parse_ffi_bool(is_ci);
    unsafe {
        SETTINGS.set_is_ci(is_ci);
        debug_print(format!("set_is_ci is_ci={:?}", SETTINGS.get_is_ci(),));
    }
}

#[no_mangle]
pub extern "C" fn set_ci_tracking_enabled(enabled: u32) {
    let enabled = parse_ffi_bool(enabled);
    unsafe {
        SETTINGS.ci_tracking_enabled = enabled;

        debug_print(format!(
            "set_ci_tracking_enabled enabled={:?}",
            SETTINGS.ci_tracking_enabled,
        ));
    }
}

#[no_mangle]
pub extern "C" fn set_user_id(user_id: *const c_char) {
    if user_id.is_null() {
        // Silently return
        return;
    }

    let user_id = parse_ffi_str(user_id);
    unsafe {
        SETTINGS.set_user_id(user_id.as_str());
        debug_print(format!("set_user_id user_id={:?}", SETTINGS.get_user_id()))
    }
}

#[no_mangle]
pub extern "C" fn set_invocation_id(invocation_id: *const c_char) {
    if invocation_id.is_null() {
        // Silently return
        return;
    }

    let invocation_id = parse_ffi_str(invocation_id);
    unsafe {
        SETTINGS.set_invocation_id(invocation_id.as_str());
        debug_print(format!(
            "set_invocation_id invocation_id={:?}",
            SETTINGS.get_invocation_id()
        ))
    }
}
