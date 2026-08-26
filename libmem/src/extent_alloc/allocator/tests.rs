use core::{cmp::Ordering::{Equal, Less}, num::NonZero};

use libtestrand::TestRng;

use alloc::vec::Vec;

use crate::{AlignedAddress, AlignedNonNull, Alignment, extent_alloc::{Extent, ExtentAllocator, RawExtent, TakenExtent, layout::{ExtentLayout, LayoutDescriptor}}};



#[test]
fn init() {

    let mut rand = TestRng::new();

    for _ in 0..2000 {

        let extent_alloc: ExtentAllocator<1024, RawExtent<1024>, ExtentLayout<1024>> = ExtentAllocator::uninit();

        let count = rand.next_range(..1500);

        let extents = random_extents(count, &mut rand, 1, 10240);



        let init_result = unsafe {
            extent_alloc.initialize(extents.iter().cloned())
        };

        assert!(matches!(init_result, Ok(()) ));

        let mut alloc = extent_alloc.map.lock();

        for ext in extents {

            let (map, ext) = match ext {
                TakenExtent::Free(ext) => (&mut alloc.free, ext),
                TakenExtent::Used(ext) => {
                    (&mut alloc.used, ext)
                },
            };

            match map.map.get(&ext.address()) {
                Some(size) => assert!(*size == ext.size()),
                None => panic!("map does not contain address -> size"),
            }

            let by_size = match map.size_index.get(&ext.size()) {
                Some(map) => map,
                None => panic!("index does not contain size -> align"),
            };

            let by_align = match by_size.get(&ext.address().get_align()) {
                Some(map) => map,
                None => panic!("index:by_size does not contain align -> address"),
            };

            match by_align.get(&ext.address()) {
                Some(_) => {/* OK */},
                None => panic!("index:by_align does not contains correct address"),
            }

        }

    }

}



#[test]
fn find_suitable_in() {

    let mut rand = TestRng::new();

    for _ in 0..4000 {

        let count = rand.next_range(1..10);
        let exts = random_extents(count, &mut rand, 128, 256);

        let allocator: ExtentAllocator<1024> = ExtentAllocator::uninit();
        assert!(matches!(unsafe { allocator.initialize(exts.iter().cloned()) }, Ok(()) ));

        let layout = ExtentLayout::from_pages(NonZero::new(rand.next_range(128..256) as u64).unwrap()).unwrap();

        let mut alloc = allocator.map.lock();


        let suitable = match super::find_suitable_in(layout.clone(), &mut alloc.free) {
            Some(ext) => ext,
            None => {   //  make sure that there is no suitable extent

                let found = exts.iter().any(|ext| {
                    match ext {
                        TakenExtent::Free(ext) => {
                            ext.size() >= layout.size()
                            && ext.address().get_align() >= layout.align()
                        },
                        _ => false,
                    }
                });

                assert!(!found, "suitable extent found manually but not by the tested function");
                continue;
            },
        };

        assert!(suitable.address().get_align() >= layout.align());
        assert!(suitable.size() >= layout.size());


        //  find the extent
        exts.iter().find(|ext| {
            match ext {
                TakenExtent::Free(ext) => ext.address() == suitable.address() && ext.size() == suitable.size(),
                _ => false,
            }
        } ).expect("suitable extent found by the tested function but not by the test");

    }

}
















fn random_free_extents(count: usize, rand: &mut TestRng, min_size: usize, max_size: usize) -> Vec<TakenExtent<1024, RawExtent<1024>>> {

    let mut vec = Vec::with_capacity(count);
    let mut address = NonZero::new(1).unwrap();

    for _ in 0..count {
        vec.push(TakenExtent::Free(random_ext(rand, &mut address, min_size, max_size)));
    }

    return vec;

    fn random_ext(rand: &mut TestRng, addr: &mut NonZero<u64>, min_size: usize, max_size: usize) -> RawExtent<1024> {
        let pages = rand.next_range(min_size..max_size) as u64;
        let ret = RawExtent::new(
            AlignedNonNull::new_up(*addr),
            NonZero::new(pages).unwrap());
        *addr = addr.saturating_add(pages.saturating_mul(1024));
        ret
    }

}



/// Generates a vector of random extents
fn random_extents(count: usize, rand: &mut TestRng, min_size: usize, max_size: usize) -> Vec<TakenExtent<1024, RawExtent<1024>>> {

    let mut vec = Vec::with_capacity(count);
    let mut address = NonZero::new(1).unwrap();

    for _ in 0..count {
        if rand.next() & 1 != 0 {
            vec.push(TakenExtent::Used(random_ext(rand, &mut address, min_size, max_size)));
        } else {
            vec.push(TakenExtent::Free(random_ext(rand, &mut address, min_size, max_size)));
        }
    }

    return vec;

    fn random_ext(rand: &mut TestRng, addr: &mut NonZero<u64>, min_size: usize, max_size: usize) -> RawExtent<1024> {
        let pages = rand.next_range(min_size..max_size) as u64;
        let ret = RawExtent::new(
            AlignedNonNull::new_up(*addr),
            NonZero::new(pages).unwrap());
        *addr = addr.saturating_add(pages.saturating_mul(1024));
        ret
    }

}
