//! The arm64 guest's `libvulkan.so` (docs/vr/dynarmic-design.md §3.2, M5).
//!
//! **Disconnected with the rest of the Vulkan backend in this fork.** The
//! guest library used to export `vkGetInstanceProcAddr` over Cordial's
//! native one, and every pointer it handed back was a stub made for that
//! command from its generated signature (`guest_vk_table`). None of that
//! exists now: the native virtual library is never registered (see
//! `symtab::build`), so there is no `vkGetInstanceProcAddr` to wrap, and the
//! guest's own `dlopen` of `libvulkan.so` fails the same way the phone
//! build's does.
//!
//! What remains is the sentinel, so the VR path — which reaches for Vulkan
//! through this module — fails loudly and greppably rather than silently.
//! The one exception is [`layout_gate_cannot_compile`], which answers the
//! question it was written to answer: the layout gates are gone.

use std::sync::Arc;

use cordial_guest::{Fault, Handler, Runtime};

#[path = "guest_vk_table.rs"]
#[allow(dead_code)]
mod table;

/// The guest's `vkGetInstanceProcAddr`, over Cordial's native one.
pub fn get_instance_proc_addr(native: usize) -> Handler {
    let _ = native;
    crate::android::vulkan::vulkan_sentinel()
}

/// The host function behind a guest `vkGetInstanceProcAddr` pointer, if that
/// pointer is one of this layer's own stubs for it: Cordial's native getter.
pub(crate) fn host_gipa_for(rt: &Runtime, guest: u64) -> Option<usize> {
    let _ = (rt, guest);
    crate::android::vulkan::vulkan_sentinel()
}

/// `const VkAllocationCallbacks*` from the guest: a host copy whose five
/// functions are host entries into the guest's, made once per distinct
/// struct content and kept for the process, since an object must be
/// destroyed with an allocator compatible with the one that created it.
pub(crate) fn allocator(rt: &Arc<Runtime>, guest: u64) -> Result<u64, Fault> {
    let _ = (rt, guest);
    crate::android::vulkan::vulkan_sentinel()
}

/// Walks a create info's `pNext` chain and, if any link holds a callback,
/// returns a host copy of the chain up to the last such link with each
/// callback behind an entry; otherwise the guest's own pointer.
///
/// # Safety
///
/// `top` is null or a guest `const T*` whose chain is well formed.
pub(crate) unsafe fn translate_chain(
    rt: &Arc<Runtime>,
    cmd: &str,
    top: u64,
    keep: &mut Vec<Box<[u64]>>,
) -> Result<u64, Fault> {
    let _ = (rt, cmd, top, keep);
    crate::android::vulkan::vulkan_sentinel()
}

/// Why this machine cannot run the layout gates, here and in `guest_xr`.
///
/// Always `Some` now: the gates compared the guest's Vulkan struct layouts
/// against the host's, and with the Vulkan backend disconnected there is
/// nothing to compare. The `guest_xr` test reads this as "skip", which is
/// the honest answer.
#[cfg(test)]
pub(crate) fn layout_gate_cannot_compile() -> Option<String> {
    Some("the Vulkan backend is disconnected in this fork; the layout gate is gone".into())
}
