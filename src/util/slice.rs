use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// Forms a slice with fewer preconditions than [`core::slice::from_raw_parts_mut`].
///
/// ## Safety
/// *   The bytes `data[..layout.size()]` must be valid for reads and writes.
/// *   The bytes `data[..layout.size()]` must be contained within a single contiguous allocation.
/// *   The memory referenced by the returned slice must not be accessed through any other pointer
///     (not derived from the return value) for the duration of lifetime `'a`.
///     Both read and write accesses are forbidden.
pub unsafe fn from_raw_bytes_layout_mut<'a>(data: NonNull<MaybeUninit<u8>>, layout: Layout) -> &'a mut [MaybeUninit<u8>] {
    // SAFETY: ✔️
    //
    // "Our" preconditions:
    //  1.  Not null:                           Enforced by type (`NonNull` cannot be null)
    //  2.  Properly aligned:                   Enforced by type (`u8` is always properly aligned)
    //  3.  Properly initialized:               Enforced by type (`MaybeUninit` is always properly initialized)
    //  4.  No larger than isize::MAX bytes:    Enforced by type (`Layout` enforces this as a precondition for `.size()`)
    //  5.  Single allocation:                  As documented in `## Safety`
    //  6.  Valid for read and write:           As documented in `## Safety`
    //  7.  For at least `layout.size()` bytes  As documented in `## Safety`
    //  8.  Lifetime access:                    As documented in `## Safety`
    //
    // Fulfills these documented preconditions:
    //  •   `data` must be valid for both reads and writes[6] for `len * mem::size_of::<T>()` many bytes[7], and it must be properly aligned[2]. This means in particular:
    //      ○   The entire memory range of this slice must be contained within a single allocated object![5] Slices can never span across multiple allocated objects.
    //      ○   `data` must be non-null[1] and aligned[2] even for zero-length slices. One reason for this is that enum layout optimizations may rely on references (including slices of any length) being aligned and non-null to distinguish them from other data. You can obtain a pointer that is usable as data for zero-length slices using NonNull::dangling().
    //  •   `data` must point to `len` consecutive[5][7] properly initialized values of type `T`[3].
    //  •   The memory referenced by the returned slice must not be accessed through any other pointer (not derived from the return value) for the duration of lifetime `'a`. Both read and write accesses are forbidden.[8]
    //  •   The total size `len * mem::size_of::<T>()` of the slice must be no larger than `isize::MAX`[4]. See the safety documentation of pointer::offset.
    //
    unsafe { core::slice::from_raw_parts_mut(data.as_ptr(), layout.size()) }
}



/// Forms a slice with fewer preconditions than [`core::slice::from_raw_parts`].
///
/// ## Safety
/// *   The bytes `data[..layout.size()]` must be valid for reads.
/// *   The bytes `data[..layout.size()]` must be contained within a single contiguous allocation.
/// *   The memory referenced by the returned slice must not be mutated for the duration of lifetime `'a`, except inside an `UnsafeCell`.
pub unsafe fn from_raw_bytes_layout<'a>(data: NonNull<MaybeUninit<u8>>, layout: Layout) -> &'a [MaybeUninit<u8>] {
    // SAFETY: ✔️
    //
    // "Our" preconditions:
    //  1.  Not null:                           Enforced by type (`NonNull` cannot be null)
    //  2.  Properly aligned:                   Enforced by type (`u8` is always properly aligned)
    //  3.  Properly initialized:               Enforced by type (`MaybeUninit` is always properly initialized)
    //  4.  No larger than isize::MAX bytes:    Enforced by type (`Layout` enforces this as a precondition for `.size()`)
    //  5.  Single allocation:                  As documented in `## Safety`
    //  6.  Valid for read:                     As documented in `## Safety`
    //  7.  For at least `layout.size()` bytes  As documented in `## Safety`
    //  8.  Lifetime access:                    As documented in `## Safety`
    //
    // Fulfills these documented preconditions:
    //  •   `data` must be valid for reads[6] for `len * mem::size_of::<T>()` many bytes[7], and it must be properly aligned[2]. This means in particular:
    //      ○   The entire memory range of this slice must be contained within a single allocated object![5] Slices can never span across multiple allocated objects.
    //      ○   `data` must be non-null[1] and aligned[2] even for zero-length slices. One reason for this is that enum layout optimizations may rely on references (including slices of any length) being aligned and non-null to distinguish them from other data. You can obtain a pointer that is usable as data for zero-length slices using NonNull::dangling().
    //  •   `data` must point to `len` consecutive[5][7] properly initialized values of type `T`[3].
    //  •   The memory referenced by the returned slice must not be mutated for the duration of lifetime 'a, except inside an UnsafeCell[8].
    //  •   The total size `len * mem::size_of::<T>()` of the slice must be no larger than `isize::MAX`[4]. See the safety documentation of pointer::offset.
    //
    unsafe { core::slice::from_raw_parts(data.as_ptr().cast_const(), layout.size()) }
}
