//! Roblox's Vulkan path, interposed onto the host loader.
//!
//! **This fork does not have one.** The virtual `libvulkan.so` and
//! `libvulkan.so.1` sonames are never registered — see `symtab::build` — so
//! the engine's `dlopen("libvulkan.so")` fails and it takes its own
//! documented fall-through to GLES3. That is the whole of the graphics
//! backend decision now, and it is baked in: no environment variable, flag
//! or setting offers Vulkan, because the interposition this module used to
//! carry is gone.
//!
//! What remains is the sentinel. Every entry point that used to reach for
//! Vulkan — the loader, the swapchain, the present modes, the capture — now
//! prints the marker and panics, so a code path that still tries to use the
//! disconnected backend fails loudly and greppably instead of silently
//! doing nothing. The one exception is [`last_extent`], which answers
//! `(0, 0)`: it is a diagnostic field in the `cordial_info` output, and
//! "there is no Vulkan extent" is the truth rather than a lie.
//!
//! The history — what this file used to do, and why each part existed — is
//! in git. The short version: the engine `dlopen`s Vulkan rather than
//! linking it, so Cordial answered that `dlopen` with a virtual library
//! exporting `vkGetInstanceProcAddr`, and interposed the handful of calls
//! that needed translating onto a desktop loader (the Android surface, the
//! swapchain's present mode, the Wayland extent). All of that is
//! disconnected, and the measurements that justified it went with it.

use std::ffi::{c_char, c_void, CStr};

/// The sonames Roblox tries, in the order it tries them.
///
/// Kept as data because `guest_link` still names them when it describes the
/// guest's library table. They are never registered — see `symtab::build` —
/// and are listed here so the disconnect has one place to name what is being
/// refused.
pub const LIBRARY_NAMES: [&str; 2] = ["libvulkan.so", "libvulkan.so.1"];

/// The sentinel every disconnected entry point fires.
///
/// The word is deliberate and is the string to grep for: a code path that
/// reaches for Vulkan in this fork has reached an impossible state, and it
/// should be obvious in a log before it is obvious anywhere else. The
/// `[instr]` tag matches the temporary-instrumentation convention this file
/// used for exactly this kind of "this should not be happening" line.
pub(crate) fn vulkan_sentinel() -> ! {
    for _ in 0..8 {
        eprintln!("[instr] FART FART FART FART FART FART FART FART FART FART FART FART FART FART FART FART");
    }
    panic!("the Vulkan backend is disconnected in this fork; this code path must not be reached");
}

/// The one export the virtual `libvulkan.so`/`libvulkan.so.1` libraries used
/// to need. `None` when the host had no Vulkan at all; the address of the
/// interposed `vkGetInstanceProcAddr` otherwise.
///
/// Now always the sentinel: nothing registers these sonames, so there is no
/// instance proc addr to hand out.
pub fn get_instance_proc_addr_symbol() -> Option<*mut c_void> {
    vulkan_sentinel()
}

/// Ask for the next presented frame to be written to `path`, and wait for it.
///
/// The sentinel: capturing a frame meant reading the Vulkan swapchain, and
/// there is no swapchain any more.
pub fn request_capture(path: &str) -> Result<String, String> {
    let _ = path;
    vulkan_sentinel()
}

/// The extent of the most recent swapchain, for callers reporting state.
///
/// `(0, 0)`, always: there is no swapchain, because there is no Vulkan. The
/// callers are diagnostics (`cordial_info`, the `updatesurface` verb), and
/// the honest answer to "how big was the last Vulkan swapchain" is "there
/// was not one".
pub fn last_extent() -> (u32, u32) {
    (0, 0)
}

/// An instance-level command from the host loader, against the real host
/// instance.
pub(crate) fn instance_fn(name: &CStr) -> Option<*mut c_void> {
    let _ = name;
    vulkan_sentinel()
}

/// The host's `vkGetDeviceProcAddr`, so device-level lookups can be forwarded
/// after the counted ones are peeled off.
pub(crate) fn device_proc_getter() -> Option<extern "C" fn(u64, *const c_char) -> *mut c_void> {
    vulkan_sentinel()
}

/// The capture for a frame that is never presented: an OpenXR engine hands
/// its images to the runtime instead (`guest_xr`).
#[cfg(target_arch = "x86_64")]
pub(crate) fn capture_image(
    instance: u64,
    physical_device: u64,
    queue_index: u32,
    target: super::capture::Target,
) {
    let _ = (instance, physical_device, queue_index, target);
    vulkan_sentinel()
}

/// The physical device `vkCreateDevice` was last called with, and the device
/// the engine's own path recorded, for the OpenXR bridge to check that the
/// handles the runtime hands back are the ones this layer saw created.
#[cfg(target_arch = "x86_64")]
pub(crate) fn created_device() -> (u64, u64) {
    vulkan_sentinel()
}

/// Recreate the window swapchain at `width`x`height` for the XR mirror.
pub(super) fn recreate_mirror_swapchain(
    device: u64,
    old: u64,
    width: u32,
    height: u32,
) -> Result<u64, String> {
    let _ = (device, old, width, height);
    vulkan_sentinel()
}
