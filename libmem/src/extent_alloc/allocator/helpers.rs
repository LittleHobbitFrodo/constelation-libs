
use alloc::collections::{BTreeMap, BTreeSet};

use crate::{AlignedAddress, AlignedNonNull};
use core::num::NonZero;
use core::sync::atomic::Ordering::Acquire;
use core::sync::atomic::{AtomicU64, Ordering::{Relaxed}};

pub(crate) struct ExtentAlloc<const ALIGN: usize> {
    /// A map of used extents
    pub(crate) used: AllocMap<ALIGN>,
    /// A map of free/unused extents
    pub(crate) free: AllocMap<ALIGN>,
}

impl<const ALIGN: usize> ExtentAlloc<ALIGN> {
    pub const fn uninit() -> Self {
        Self { used: AllocMap::uninit(), free: AllocMap::uninit() }
    }
}


/// A "side" of the extent allocator
pub(crate) struct AllocMap<const ALIGN: usize> {
    /// Resolves address to a extent size
    pub(crate) map: BTreeMap<AlignedNonNull<NonZero<u64>, ALIGN>, NonZero<u64>>,
    /// Resolves extent size -> extent address alignment -> extent address
    pub(crate) size_index: BTreeMap<NonZero<u64>, BTreeMap<NonZero<u64>, BTreeSet<AlignedNonNull<NonZero<u64>, ALIGN>>>>,
}

impl<const ALIGN: usize> AllocMap<ALIGN> {
    pub const fn uninit() -> Self {
        Self { map: BTreeMap::new(), size_index: BTreeMap::new() }
    }
}

/*pub(crate) struct ExtentAllocInner<const ALIGN: usize> {
    /// Manages used frames
    pub(crate) used: Couple<ALIGN>,
    /// Manages unused frames
    pub(crate) free: Couple<ALIGN>,
}

impl<const ALIGN: usize> ExtentAllocInner<ALIGN> {
    pub(crate) const fn uninit() -> Self {
        Self {
            used: Couple::uninit(),
            free: Couple::uninit()
        }
    }
}


pub(crate) struct Couple<const ALIGN: usize> {
    /// Used to determine physical neighbors
    /// - Key: Physical address
    /// - Value: Block size
    pub(crate) map: BTreeMap<AlignedAddress<u64, ALIGN>, NonZero<u64>>,
    /// Used to find the smalled available block efficiently
    /// - Key: Block size
    /// - Value: Addresses of all blocks sharing th size
    pub(crate) size_index: BTreeMap<NonZero<u64>, BTreeSet<AlignedAddress<u64, ALIGN>>>,
}

impl<const ALIGN: usize> Couple<ALIGN> {
    pub(crate) const fn uninit() -> Self { Self { map: BTreeMap::new(), size_index: BTreeMap::new() } }
}*/





/// Tracks how many pages are free, used, and present in memory
pub struct Stats {
    /// Total usable memory pages located in physical memory
    total: AtomicU64,
    /// Count of all used pages
    used: AtomicU64,
}


impl Stats {

    pub(crate) const fn uninit() -> Self {
        Self { total: AtomicU64::new(0), used: AtomicU64::new(0) }
    }

    /// Total usable memory pages located in physical memory
    #[inline(always)]
    pub fn total(&self) -> &AtomicU64 { &self.total }

    /// Count of all used pages
    #[inline(always)]
    pub fn used(&self) -> &AtomicU64 { &self.used }

    /// Count of all free pages
    #[inline(always)]
    pub fn get_free(&self) -> u64 {
        self.get_total().saturating_sub(self.used().load(Acquire))
    }

    /// Returns the total count of all pages present in physical memory
    #[inline(always)]
    pub fn get_total(&self) -> u64 { self.total.load(Relaxed) }

    /// Returns the count of used pages
    /// - This operation uses the `Relaxed` ordering so the
    /// returned value should be considered out of date
    #[inline(always)]
    pub fn get_used(&self) -> u64 { self.used.load(Relaxed) }
}
