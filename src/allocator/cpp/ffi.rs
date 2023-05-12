#![allow(non_camel_case_types)]

#[cfg(cpp17)] type align_val_t = usize;
type size_t         = usize;
type void           = ();

extern "C" {
    /// Binds to:
    /// ```cpp
    /// try {
    ///     return std::allocator<char>().allocate(count);
    /// } catch (const std::bad_alloc&) {
    ///     return nullptr;
    /// }
    /// ```
    #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "std_allocator_char_allocate"        )] pub fn std_allocator_char_allocate(count: usize) -> *mut void;

    /// `std::allocator<char>().deallocate(ptr, count);`
    #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "std_allocator_char_deallocate"      )] pub fn std_allocator_char_deallocate(ptr: *mut void, count: size_t);

    #[doc = "`::operator new(count, std::nothrow)`"     ] #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_new_nothrow"               )] pub fn operator_new_nothrow       (count: size_t) -> *mut void;
    #[doc = "`::operator new[](count, std::nothrow)`"   ] #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_new_array_nothrow"         )] pub fn operator_new_array_nothrow (count: size_t) -> *mut void;
    #[doc = "`::operator delete(ptr);`"                 ] #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_delete"                    )] pub fn operator_delete            (ptr: *mut void);
    #[doc = "`::operator delete[](ptr);`"               ] #[cfg(cpp98)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_delete_array"              )] pub fn operator_delete_array      (ptr: *mut void);

    // C++17+
    #[doc = "`::operator new(count, align, std::nothrow)`"      ] #[cfg(cpp17)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_new_align_nothrow"         )] pub fn operator_new_align_nothrow       (count: size_t, align: align_val_t) -> *mut void;
    #[doc = "`::operator new[](count, align, std::nothrow)`"    ] #[cfg(cpp17)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_new_array_align_nothrow"   )] pub fn operator_new_array_align_nothrow (count: size_t, align: align_val_t) -> *mut void;
    #[doc = "`::operator delete(ptr, align);`"                  ] #[cfg(cpp17)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_delete_align"              )] pub fn operator_delete_align            (ptr: *mut void, align: align_val_t);
    #[doc = "`::operator delete[](ptr, align);`"                ] #[cfg(cpp17)] #[link_name = concat!(env!("IALLOC_PREFIX"), "operator_delete_array_align"        )] pub fn operator_delete_array_align      (ptr: *mut void, align: align_val_t);
}
