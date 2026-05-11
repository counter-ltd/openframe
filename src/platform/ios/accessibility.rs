//! VoiceOver snapshot: hitboxes with [`crate::AccessibilityProperties`] exposed via FFI.

use parking_lot::Mutex;

struct StoredNode {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    traits: u64,
    label: String,
    hint: Option<String>,
}

static IOS_ACCESSIBILITY_NODES: Mutex<Vec<StoredNode>> = Mutex::new(Vec::new());

pub(crate) fn sync_from_hitboxes(hitboxes: &[crate::Hitbox]) {
    let mut next = Vec::new();
    for hb in hitboxes.iter().rev() {
        let Some(props) = hb.accessibility.as_ref() else {
            continue;
        };
        let bounds = hb.bounds;
        next.push(StoredNode {
            x: bounds.origin.x.into(),
            y: bounds.origin.y.into(),
            w: bounds.size.width.into(),
            h: bounds.size.height.into(),
            traits: props.traits,
            label: props.label.to_string(),
            hint: props.hint.as_ref().map(|h| h.to_string()),
        });
    }
    *IOS_ACCESSIBILITY_NODES.lock() = next;
}

/// Number of nodes in the last synced accessibility snapshot (after each frame draw).
pub fn snapshot_node_count() -> usize {
    IOS_ACCESSIBILITY_NODES.lock().len()
}

fn write_nul_terminated_utf8_slice(dest: &mut [u8], text: &str) -> usize {
    if dest.is_empty() {
        return 0;
    }
    let max_body = dest.len().saturating_sub(1);
    let bytes = text.as_bytes();
    let n = bytes.len().min(max_body);
    dest[..n].copy_from_slice(&bytes[..n]);
    dest[n] = 0;
    n
}

/// Writes rectangle (`x`, `y`, `width`, `height`), traits, and NUL-terminated UTF-8 label/hint.
///
/// # Safety
///
/// `out_rect4` must point to four `f32` values when non-null. When label/hint buffers are non-null
/// and capacity includes space for a trailing NUL, strings are truncated to fit.
pub unsafe fn snapshot_query_node(
    index: usize,
    out_rect4: *mut f32,
    out_traits: *mut u64,
    label_buf: *mut u8,
    label_cap: usize,
    hint_buf: *mut u8,
    hint_cap: usize,
    out_label_len: *mut usize,
    out_hint_len: *mut usize,
) {
    if out_traits.is_null() || out_label_len.is_null() || out_hint_len.is_null() {
        return;
    }
    let guard = IOS_ACCESSIBILITY_NODES.lock();
    let Some(node) = guard.get(index) else {
        if !out_rect4.is_null() {
            unsafe {
                std::ptr::write_bytes(out_rect4, 0, 4);
            }
        }
        unsafe {
            out_traits.write(0);
            out_label_len.write(0);
            out_hint_len.write(0);
        }
        return;
    };

    if !out_rect4.is_null() {
        unsafe {
            let r = std::slice::from_raw_parts_mut(out_rect4, 4);
            r[0] = node.x;
            r[1] = node.y;
            r[2] = node.w;
            r[3] = node.h;
        }
    }
    unsafe {
        out_traits.write(node.traits);
    }

    let label_len = if label_buf.is_null() || label_cap == 0 {
        0
    } else {
        unsafe {
            let slice = std::slice::from_raw_parts_mut(label_buf, label_cap);
            write_nul_terminated_utf8_slice(slice, &node.label)
        }
    };
    unsafe {
        out_label_len.write(label_len);
    }

    let hint_str = node.hint.as_deref().unwrap_or("");
    let hint_len = if hint_buf.is_null() || hint_cap == 0 {
        0
    } else {
        unsafe {
            let slice = std::slice::from_raw_parts_mut(hint_buf, hint_cap);
            write_nul_terminated_utf8_slice(slice, hint_str)
        }
    };
    unsafe {
        out_hint_len.write(hint_len);
    }
}
