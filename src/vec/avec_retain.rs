use crate::fat::Free;
use crate::vec::AVec;

use core::mem::MaybeUninit;



impl<T, A: Free> AVec<T, A> {
    pub(super) fn retain_imp<F: FnMut(&mut T) -> bool>(&mut self, mut retain: F) {
        let mut i = 0;
        loop {
            let Some(element) = self.get_mut(i) else { return };
            if !retain(element) { break }
            i += 1;
        }

        let original_len = self.len;
        self.len -= 1;
        let mut remove = RemoveHoleOnDrop { data: &mut self.data, hole: i, end: i+1 };
        unsafe { remove.data[i].assume_init_drop() };

        while remove.end < original_len {
            let element = &mut remove.data[remove.end];
            if retain(unsafe { element.assume_init_mut() }) {
                remove.data.swap(remove.hole, remove.end);
                remove.hole += 1;
                remove.end  += 1;
            } else {
                unsafe { element.assume_init_drop() };
                remove.end  += 1;
                self.len    -= 1;
            }
        }
    }
}

struct RemoveHoleOnDrop<'a, T> {
    data:       &'a mut [MaybeUninit<T>],
    hole:       usize, // index
    end:        usize, // index, >= hole
}

impl<T> Drop for RemoveHoleOnDrop<'_, T> {
    fn drop(&mut self) {
        if self.hole == self.end { return }
        let len  = self.data.len();
        let data = self.data.as_ptr();
        let data = data as *mut MaybeUninit<T>;
        unsafe { core::ptr::copy(data.add(self.end), data.add(self.hole), len - self.end) }
    }
}



#[test] fn retain() {
    let mut v = AVec::<u32, crate::allocator::alloc::Global>::new();
    v.try_extend_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    v.retain(|x| *x % 2 == 0);
    assert_eq!(v[..], [2, 4]);
}

// TODO: validate drop logic
