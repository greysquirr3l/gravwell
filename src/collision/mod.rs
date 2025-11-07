//! Collision detection and handling (requires std feature).

#[cfg(feature = "std")]
pub mod spatial_hash;

#[cfg(feature = "std")]
pub mod aabb;
