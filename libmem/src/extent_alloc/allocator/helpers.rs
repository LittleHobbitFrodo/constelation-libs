
use alloc::collections::{BTreeMap, BTreeSet};

use crate::AlignedAddress;
use core::num::NonZero;
use core::sync::atomic::{AtomicUsize, Ordering::{Relaxed}};

pub(crate) struct Map<const ALIGN: usize> {
    /// Manages used frames
    pub(crate) used: Couple<ALIGN>,
    /// Manages unused frames
    pub(crate) free: Couple<ALIGN>,
}

impl<const ALIGN: usize> Map<ALIGN> {
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
    pub(crate) map: BTreeMap<AlignedAddress<usize, ALIGN>, NonZero<usize>>,
    /// Used to find the smalled available block efficiently
    /// - Key: Block size
    /// - Value: Addresses of all blocks sharing th size
    pub(crate) size_index: BTreeMap<NonZero<usize>, BTreeSet<AlignedAddress<usize, ALIGN>>>,
}

impl<const ALIGN: usize> Couple<ALIGN> {
    pub(crate) const fn uninit() -> Self { Self { map: BTreeMap::new(), size_index: BTreeMap::new() } }
}


/// Tracks how many pages are free, used, and present in memory
pub struct Stats {
    /// Total usable memory pages located in physical memory
    total: AtomicUsize,
    /// Count of all used pages
    used: AtomicUsize,
}


impl Stats {

    pub(crate) const fn uninit() -> Self {
        Self { total: AtomicUsize::new(0), used: AtomicUsize::new(0) }
    }

    /// Total usable memory pages located in physical memory
    #[inline(always)]
    pub fn total(&self) -> &AtomicUsize { &self.total }

    /// Count of all used pages
    #[inline(always)]
    pub fn used(&self) -> &AtomicUsize { &self.used }

    /// Returns the total count of all pages present in physical memory
    #[inline(always)]
    pub fn get_total(&self) -> usize { self.total.load(Relaxed) }

    /// Returns the count of used pages
    /// - This operation uses the `Relaxed` ordering so the
    /// returned value should be considered out of date
    #[inline(always)]
    pub fn get_used(&self) -> usize { self.used.load(Relaxed) }
}
