//! ETC2/EAC emulation where the driver lacks the feature (ADR-049).
//!
//! **Gone with the Vulkan disconnect.** This module answered a physical
//! device whose `textureCompressionETC2` was false by reporting the feature
//! set and decoding the engine's ETC2/EAC images on the CPU. It reached for
//! the host Vulkan loader through [`super::vulkan`], which is disconnected,
//! so there is no device to query and no image to decode.
//!
//! The file remains as the place the emulation used to live. The decoder it
//! called is [`super::etc_decode`], which is pure and stays.
