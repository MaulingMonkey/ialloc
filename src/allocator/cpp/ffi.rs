#![allow(non_camel_case_types)]

#[cfg(cpp17)] type align_val_t = usize;
type size_t         = usize;
type void           = ();

extern "C" {
    #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "std_allocator_char_allocate"        )] pub fn std_allocator_char_allocate(count: usize) -> *mut void;
    #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "std_allocator_char_deallocate"      )] pub fn std_allocator_char_deallocate(ptr: *mut void, count: size_t);

    #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_new_nothrow"               )] pub fn operator_new_nothrow       (count: size_t) -> *mut void;
    #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_new_array_nothrow"         )] pub fn operator_new_array_nothrow (count: size_t) -> *mut void;
    #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_delete"                    )] pub fn operator_delete            (ptr: *mut void);
    #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_delete_array"              )] pub fn operator_delete_array      (ptr: *mut void);

    // C++17+
    #[cfg(cpp17)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_new_align_nothrow"         )] pub fn operator_new_align_nothrow       (count: size_t, align: align_val_t) -> *mut void;
    #[cfg(cpp17)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_new_array_align_nothrow"   )] pub fn operator_new_array_align_nothrow (count: size_t, align: align_val_t) -> *mut void;
    #[cfg(cpp17)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_delete_align"              )] pub fn operator_delete_align            (ptr: *mut void, align: align_val_t);
    #[cfg(cpp17)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_delete_array_align"        )] pub fn operator_delete_array_align      (ptr: *mut void, align: align_val_t);
}
