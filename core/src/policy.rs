//! Defines some compile-time configuration
//!
//! A policy is a way to define how do you want to represents your entity `Id` et `Version`
//! in memory. You can choose between different unsigned interger sizes that will determine
//! a maximum of entities that can exists at the same time.
//!

#[cfg(feature = "u16_handle")]
pub use self::u16_handle::*;

#[cfg(feature = "u32_handle")]
pub use self::u32_handle::*;

pub type IdSet = ::bit_set::BitSet;

#[cfg(feature = "u16_handle")]
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

#[cfg(feature = "u32_handle")]
mod u32_handle {
    /// The id type for an entity
    pub type Id = u32;

    /// The version type for an entity
    pub type Version = u32;

    /// Returns the maxmimum of entities that can exists at the same time
    pub fn max_entity_count() -> usize {
        use std::u32::MAX;
        return MAX as usize;
    }

    /// Converts a usize to an Id.
    ///
    /// **Panics** if the usize overflows the `Id`
    #[inline]
    pub fn id_from_usize(value: usize) -> Id {
        value as u32
    }
}
