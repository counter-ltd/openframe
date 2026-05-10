//! [`UIPasteboard`] bridging for [`crate::ClipboardItem`].
use crate::ClipboardItem;
use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use std::{
    ffi::{CStr, CString},
    path::PathBuf,
    ptr,
};

/// Writes text-rich clipboard entries to the general pasteboard.
pub(crate) fn write_to_uikit_pasteboard(item: &ClipboardItem) {
    if let Some(text) = item.text() {
        unsafe {
            let pb: *mut Object = msg_send![class!(UIPasteboard), generalPasteboard];
            let ns = ns_string(&text);
            if !ns.is_null() {
                let _: () = msg_send![pb, setString: ns];
            }
        }
    }
}

/// Reads a plain string from the general pasteboard, if any.
pub(crate) fn read_from_uikit_pasteboard() -> Option<ClipboardItem> {
    unsafe {
        let pb: *mut Object = msg_send![class!(UIPasteboard), generalPasteboard];
        let s: *mut Object = msg_send![pb, string];
        if s.is_null() {
            return None;
        }
        nsstring_to_string(s).map(ClipboardItem::new_string)
    }
}

/// Path of the running app bundle (`NSBundle.mainBundle.bundlePath`), when linked with UIKit.
pub(crate) fn main_bundle_path() -> anyhow::Result<PathBuf> {
    unsafe {
        let bundle: *mut Object = msg_send![class!(NSBundle), mainBundle];
        let path: *mut Object = msg_send![bundle, bundlePath];
        if path.is_null() {
            anyhow::bail!("NSBundle.bundlePath returned nil");
        }
        nsstring_to_string(path)
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("could not read bundle path"))
    }
}

unsafe fn ns_string(s: &str) -> *mut Object {
    let Ok(c) = CString::new(s) else {
        return ptr::null_mut();
    };
    let Some(ns_string_cls) = objc::runtime::Class::get("NSString") else {
        return ptr::null_mut();
    };
    msg_send![ns_string_cls, stringWithUTF8String: c.as_ptr()]
}

unsafe fn nsstring_to_string(ns: *mut Object) -> Option<String> {
    let utf8: *const i8 = msg_send![ns, UTF8String];
    if utf8.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(utf8) }
        .to_str()
        .ok()
        .map(ToString::to_string)
}
