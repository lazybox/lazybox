//! Defines some compile-time configuration
//!
//! A policy is a way to define how do you want to represents your entity `Id` et `Version`
//! in memory. You can choose between different unsigned interger sizes that will determine
//! a maximum of entities that can exists at the same time.
//!

#[cfg(feature = "u16_handle")]
pub use self::u16_handle::*;

mod u16_handle {
    /// The id type for an entity
    pub type Id = u16;

    /// The version type for an entity
    pub type Version = u16;

    /// Returns the maxmimum of entities that can exists at the same time
    pub fn max_entity_count() -> usize {
        use std::u16::MAX;
        return MAX as usize;
    }

    /// Converts a usize to an Id.
    ///
    /// **Panics** if the usize overflows the `Id`
    #[inline]
    pub fn id_from_usize(value: usize) -> Id {
        value as u16
    }
}
