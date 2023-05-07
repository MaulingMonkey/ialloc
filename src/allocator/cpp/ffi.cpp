#include <memory>
#include <new>

using std::size_t;
using std::nothrow;

#define IALLOC_CONCAT2(a,b) a ## b
#define IALLOC_CONCAT(a,b) IALLOC_CONCAT2(a, b)
#define IALLOC_FN(ret, name) extern "C" ret IALLOC_CONCAT(IALLOC_PREFIX, name)



template < typename T > struct ebco : T { char ch; };
static_assert(sizeof(ebco<std::allocator<char>>) == 1, "std::allocator<char> contains members/data, not interchangeable?");
IALLOC_FN(void*, std_allocator_char_allocate        ) (size_t count)                    { return std::allocator<char>().allocate(count); }
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
