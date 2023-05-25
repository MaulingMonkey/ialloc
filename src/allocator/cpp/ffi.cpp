#include <memory>
#include <new>

using std::size_t;
using std::nothrow;

#define IALLOC_CONCAT2(a,b) a ## b
#define IALLOC_CONCAT(a,b) IALLOC_CONCAT2(a, b)
#define IALLOC_FN(ret, name) extern "C" ret IALLOC_CONCAT(IALLOC_PREFIX, name)



#if __cpp_static_assert
    // validate std::allocator<char> is stateless
    #if __cpp_lib_allocator_traits_is_always_equal
        static_assert(std::allocator_traits<std::allocator<char> >::is_always_equal::value, "std::allocator<char> isn't stateless");
    #else
        template < typename T > struct ebco : T { char ch; };
        static_assert(sizeof(ebco<std::allocator<char>>) == 1, "std::allocator<char> contains members/data, not interchangeable?");
    #endif
#endif

#if defined(_MSC_VER)
#   if !defined(_MT)
#       if _MSC_VER >= 1400
#           error "Rust bindings assume the C++ standard library is thread safe, but you're supposedly using a variant that isn't.  You're using Visual Studio 2005 or later according to _MSC_VER, which supposedly removed single-threaded variants of the C++ standard library.  This is likely some kind of weirdly misconfigured build environment that undefines _MT, or fails to define it in the first place."
#       else
#           error "Rust bindings assume the C++ standard library is thread safe, but you're supposedly using a variant that isn't.  You're supposedly using Visual Studio 2003 or earlier.  Consider using a compiler that's not two decades old.  Alternative, switch to a multithreaded runtime with /MD, /MDd, /MT, or /MTd.  See https://learn.microsoft.com/en-us/cpp/build/reference/md-mt-ld-use-run-time-library for details"
#       endif
#   endif
#endif

IALLOC_FN(void*, std_allocator_char_allocate        ) (size_t count)                    { try { return std::allocator<char>().allocate(count); } catch (const std::bad_alloc&) { return 0; } }
IALLOC_FN(void,  std_allocator_char_deallocate      ) (char* ptr, size_t count)         { return std::allocator<char>().deallocate(ptr, count); }

IALLOC_FN(void*, operator_new_nothrow               ) (size_t count)                    { return ::operator new  (count,        nothrow); }
IALLOC_FN(void*, operator_new_array_nothrow         ) (size_t count)                    { return ::operator new[](count,        nothrow); }
IALLOC_FN(void, operator_delete                     ) (void* ptr)                       { return ::operator delete  (ptr); }
IALLOC_FN(void, operator_delete_array               ) (void* ptr)                       { return ::operator delete[](ptr); }

#if __cpp_aligned_new
using std::align_val_t;
IALLOC_FN(void*, operator_new_align_nothrow         ) (size_t count, align_val_t align) { return ::operator new  (count, align, nothrow); }
IALLOC_FN(void*, operator_new_array_align_nothrow   ) (size_t count, align_val_t align) { return ::operator new[](count, align, nothrow); }
IALLOC_FN(void,  operator_delete_align              ) (void* ptr, align_val_t align)    { return ::operator delete  (ptr, align); }
IALLOC_FN(void,  operator_delete_array_align        ) (void* ptr, align_val_t align)    { return ::operator delete[](ptr, align); }
#endif // __cpp_aligned_new
