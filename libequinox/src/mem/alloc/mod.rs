use core::num::NonZero;
use core::ptr::NonNull;
use core::sync::atomic::Ordering::{self, Acquire, SeqCst};
use core::sync::atomic::AtomicPtr;


mod stack;
mod page_tables;


/// Raw memory layout of any allocation request
/// - What the kernel declares and bootloader fills
///
/// How it works:
/// 1. The kernel creates request to allocation of certain kind
/// 2. Upon boot, the bootloader fills the data in it
///     - Allocates the response and writes its address into the request
/// 3. The kernel reads the response
///
/// Misc. notes:
/// - All the responses are guaranteed to be
/// located in `BootloaderReclaimable` memory,
/// so the kernel does not need to worry about it
#[repr(C, align(64))]
struct RawAllocRequest {
    /// Verification code (per request)
    /// - Set by kernel, verified by the bootloader
    code: [u8; 24],
    /// Revision (version of the API)
    revision: NonZero<u64>,
    /// Reference to the response (filled by the bootloader)
    /// - For bootloader: If non-null the request has been initialized
    /// - For kernel: If null, the response has already been read
    response: AtomicPtr<AllocationResponse>,
}

unsafe impl Sync for RawAllocRequest {}

impl RawAllocRequest {

    /// Atomically checks whether the request has been initialized
    #[inline]
    pub fn is_initialized(&self) -> bool {
        !self.response.load(Ordering::Acquire).is_null()
    }


    /// Returns the revision (API version)
    #[inline]
    pub fn revision(&self) -> NonZero<u64> { self.revision }

}

/// The response to the `AllocationRequest` allocated and managed by the bootloader
///
/// # How to use
/// - The allocated response must be contained in memory marked as bootloader reclaimable memory
///   - The kernel can reclaim the bootloader reclaimable memory after all kernel requests have been read
/// - The response cannot be deallocated or modified after its request has been initialized
#[repr(C, align(32))]
#[derive(Clone)]
pub(crate) struct AllocationResponse {
    /// Version of the response (less than or equal to the request revision)
    revision: NonZero<u64>,
    /// The virtual base address of the allocated memory
    /// - The lowest address of the allocation
    virt: NonNull<u8>,
    /// The physical base address of the allocated memory
    /// - The lowest address of the allocation
    phys: NonZero<u64>,
    /// The size of the allocation in 4KB (4096 bytes) pages
    page_count: NonZero<u64>,
    //  other fields ...
}

/// Indicates error while verifying any allocation request
pub enum VerificationError {
    /// The requesthas already been initialized
    /// - This is not a real error, but multiple initializations should not happen
    AlreadyInitialized,
    /// The entry verification code does not match the expected value
    InvalidVerificationCode,
}

pub(crate) struct AllocationRequest {
    raw: RawAllocRequest
}



impl AllocationResponse {

    /// Returns the revision used for this allocation (API version)
    #[inline]
    pub fn revision(&self) -> NonZero<u64> { self.revision }

    /// Returns the starting (also the lowest) virtual address for this allocation
    /// - The address is guaranteed to be aligned to 4KB
    #[inline]
    pub fn virtual_address(&self) -> NonNull<u8> { self.virt }

    /// Returns the starting (also the lowest) physical address for this allocation
    /// - The address is guaranteed to be aligned to 4KB
    #[inline]
    pub fn physical_address(&self) -> NonZero<u64> { self.phys }

    /// Returns the size of the allocation in 4KB pages
    #[inline]
    pub fn size_in_pages(&self) -> NonZero<u64> { self.page_count }

    /// Returns the size of the allocation in bytes
    #[inline]
    pub fn size_in_bytes(&self) -> NonZero<u64> {
        self.page_count.saturating_mul(unsafe { NonZero::new_unchecked(4096) })
    }
}

#[cfg(any(feature = "bootloader", feature = "testing"))]
pub enum ResponseCreationError {
    MisalignedVirtualAddress,
    MisalignedPhysicalAddress,
}

#[cfg(any(feature = "bootloader", feature = "testing"))]
impl AllocationResponse {

    /// Constructs new allocation response from given data
    ///
    /// Returns `Err` if `virt` or `phys` are not properly aligned to `4096`
    #[cold]
    pub(crate) fn new(revision: NonZero<u64>, virt: NonNull<u8>, phys: NonZero<u64>, page_count: NonZero<u64>) -> Result<Self, ResponseCreationError> {
        if (virt.as_ptr() as usize) & 4095 != 0 {
            return Err(ResponseCreationError::MisalignedVirtualAddress)
            //panic!("virtual address of the allocation is misaligned");
        }

        if phys.get() & 4095 != 0 {
            return Err(ResponseCreationError::MisalignedPhysicalAddress)
            //panic!("physical address of the allocation is misaligned");
        }

        Ok(Self { revision, virt, phys, page_count })
    }


    /// Tries to initialize the request, returns `Err`
    /// if the verification code does not match the expected
    /// value or if the request is already initialized
    ///
    /// # Safety
    /// The response must be allocated in memory
    /// marked as bootloader reclaimable and
    /// cannot be deallocated and/or modified after this operation
    #[inline(never)]
    #[cold]
    pub unsafe fn initialize(&self, request: NonNull<AllocationRequest>, verif_code: &'_ [u8; 24]) -> Result<(), VerificationError> {
        let raw = unsafe { &request.as_ref().raw };

        if &raw.code != verif_code {
            return Err(VerificationError::InvalidVerificationCode)
        }

        //  Write the pointer if uninit
        if let Err(_) = raw.response.compare_exchange(core::ptr::null_mut(), self as *const AllocationResponse as *mut AllocationResponse, Acquire, SeqCst) {
            return Err(VerificationError::AlreadyInitialized)
        }

        Ok(())
    }

}

impl core::fmt::Debug for AllocationResponse {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AllocationResponse {{\n    revision: {rev},\n    virt: {virt:p},\n    phys: {phys:p},\n    page_count: {count}\n}}",
            rev = self.revision.get(),
            virt = self.virt,
            phys = NonNull::new(self.phys.get() as *mut u8).unwrap(),
            count = self.page_count
        )
    }
}


impl AllocationRequest {

    #[cfg(any(feature = "kernel", feature = "testing"))]
    /// Constructs a kernel request
    pub(crate) const fn new(revision: NonZero<u64>, verif_code: &'static [u8; 24]) -> Self {
        Self {
            raw: RawAllocRequest {
                code: *verif_code,
                revision,
                response: AtomicPtr::new(core::ptr::null_mut()),
            }
        }
    }


    /// Verifies and reads the response
    /// - Returns `NonNull` pointer to the response
    /// - This will nullate the reference so the response cannot be read again
    ///
    /// Returns `Err` if the response is not present
    ///
    /// # Panics
    ///
    /// > Proper panic messages are provided
    ///
    /// - If the pointer to the response is misaligned
    /// - If response revision is greater than request revision
    /// - If virtual or physical address is not aligned to 4096
    #[inline(never)]
    #[cold]
    pub(crate) unsafe fn verify_response(&self) -> Result<NonNull<AllocationResponse>, ()> {

        const NULL: *mut AllocationResponse = core::ptr::null_mut();

        let ptr = match NonNull::new(self.raw.response.swap(core::ptr::null_mut(), Ordering::SeqCst)) {
            Some(ptr) => ptr,
            None => return Err(()),
        };

        let response = unsafe { ptr.as_ref() };

        if response.revision > self.raw.revision {
            panic!("unsupported response revision (greater than request revision)");
        }

        if (response.virt.as_ptr() as usize) & 4095 != 0 {
            panic!("misaligned allocation virtual address (must be aligned to 4096)");
        }

        if response.phys.get() & 4095 != 0 {
            panic!("misaligned allocation physical address (must be aligned to 4096)");
        }

        Ok(ptr)

    }

    /// Verifies and reads the response
    /// - This will nullate the reference so the response cannot be read again
    ///
    /// Returns `Err` if the response is not present
    ///
    /// # Panics
    ///
    /// > Proper panic messages are provided
    ///
    /// - If the pointer to the response is misaligned
    /// - If response revision is greater than request revision
    /// - If virtual or physical address is not aligned to 4096
    #[cfg(any(feature = "kernel", feature = "testing"))]
    #[inline(never)]
    #[cold]
    pub fn get_response(&self) -> Result<AllocationResponse, ()> {

        let response = unsafe { self.verify_response()?.as_ref() };

        Ok(AllocationResponse {
            revision: response.revision,
            virt: response.virt,
            phys: response.phys,
            page_count: response.page_count
        })

    }
}


/*#[cfg(feature = "testing")]
mod tests {

    use super::*;

    const VERIF_CODE: &[u8; 24] = b"SOME RANDOM TXT TO MATCH";

    #[ministd::testing("alloc-API", harness(any))]
    fn get_response() {

        let request = AllocationRequest::new(NonZero::new(1).unwrap(), VERIF_CODE);

        let response = match AllocationResponse::new(NonZero::new(1).unwrap(), NonNull::new(0xDEAD0000 as *mut u8).unwrap(), NonZero::new(4096).unwrap(), NonZero::new(1).unwrap()) {
            Ok(resp) => resp,
            Err(_) => fail!("failed to generate response"),
        };

        match unsafe { response.initialize(NonNull::from(&request), VERIF_CODE) } {
            Ok(_) => {},
            Err(e) => match e {
                VerificationError::AlreadyInitialized => fail!("ERROR: already initialized"),
                VerificationError::InvalidVerificationCode => fail!("ERROR: invalid verif. code"),
            }
        }

        let resp = match request.get_response() {
            Ok(r) => r,
            Err(_) => fail!("get_response failed"),
            //Err(_) => panic!("get_response failed"),
        };

        tassert!(resp.revision() == response.revision());
        tassert!(resp.virtual_address() == response.virtual_address());
        tassert!(resp.physical_address() == response.physical_address());
        tassert!(resp.size_in_pages() == response.size_in_pages());

    }

}*/
