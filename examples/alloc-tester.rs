extern crate std;

use std::alloc::Layout;
use std::ffi::c_char as char;

use ialloc::*;
use ialloc::allocator::*;



struct Test<A> {
    pub name:   &'static str,
    pub create: fn()->A,
    pub thin:   Option<AlignmentRange>,
    pub fat:    Option<AlignmentRange>,
}

#[derive(Clone, Copy, Debug)] struct AlignmentRange {
    pub min: Alignment,
    pub max: Alignment,
}

impl<A> Test<A> {
    pub fn new(name: &'static str, create: fn()->A) -> Self {
        Self { name, create, thin: None, fat: None }
    }

    pub fn thin(&mut self) -> &mut Self where A : thin::Alloc + thin::Free {
        let mut thin = AlignmentRange { min: Alignment::MAX, max: Alignment::MAX };
        for (dst, min_size) in [(&mut thin.min, 1), (&mut thin.max, 4096)] {
            for _ in 0 .. 100 {
                let alloc = (self.create)();
                let addrs = [(); 4096].into_iter().enumerate().map(|(i, _)| alloc.alloc_uninit(i%16+min_size).unwrap()).collect::<Vec<_>>();
                let addrbits = addrs[..].iter().copied().map(|addr| addr.as_ptr() as usize).reduce(|x,y| x|y).unwrap();
                addrs.iter().copied().for_each(|addr| unsafe { alloc.free(addr) });
                let align = Alignment::new(1 << addrbits.trailing_zeros()).unwrap();
                *dst = align.min(*dst);
            }
        }
        self.thin = Some(thin);
        self
    }

    pub fn fat(&mut self) -> &mut Self where A : fat::Alloc + fat::Free {
        let mut fat = AlignmentRange { min: Alignment::MAX, max: ALIGN_1 };

        for _ in 0 .. 100 {
            let layout = Layout::new::<u8>(); // 1B
            let alloc = (self.create)();
            let addrs = [(); 4096].map(|_| alloc.alloc_uninit(layout).unwrap());
            let addrbits = addrs[..].iter().copied().map(|addr| addr.as_ptr() as usize).reduce(|x,y| x|y).unwrap();
            addrs.iter().copied().for_each(|addr| unsafe { alloc.free(addr, layout) });
            let align = Alignment::new(1 << addrbits.trailing_zeros()).unwrap();
            fat.min = align.min(fat.min);
        }

        fat.max = fat.min;
        let alloc = (self.create)();
        while let Some(next) = fat.max.as_usize().checked_shl(1) {
            let Some(align) = Alignment::new(next) else { break };
            let Ok(layout) = Layout::from_size_align(next, next) else { break };
            let Ok(addr) = alloc.alloc_uninit(layout) else { break };
            fat.max = align;
            unsafe { alloc.free(addr, layout) };
        }

        self.fat = Some(fat);
        self
    }

    pub fn print(&self) {
        let name = self.name;
        let thin = self.thin.map_or_else(|| format!(""), |t| if t.min == t.max { format!("{:?}", t.min) } else { format!("{:?} ..= {:?}", t.min, t.max) });
        let fat  = self.fat .map_or_else(|| format!(""), |t| if t.min == t.max { format!("{:?}", t.min) } else { format!("{:?} ..= {:?}", t.min, t.max) });
        println!("{name: <25}{thin: <20}{fat: <20}");
    }
}

fn main() {
    println!("{: <25}{: <20}{: <20}", "",          "thin::Alloc",   "fat::Alloc",   );
    println!("{: <25}{: <20}{: <20}", "Allocator", "Alignment",     "Alignment",    );
    println!("{:=<65}", "");
    #[cfg(feature = "alloc")]   Test::new("Global",                 || alloc::Global                    )       .fat().print();
    #[cfg(c89)]                 Test::new("Malloc",                 || c::Malloc                        ).thin().fat().print();
    #[cfg(c89)]                 Test::new("AlignedMalloc",          || c::AlignedMalloc                 )       .fat().print();
    #[cfg(cpp98)]               Test::new("NewDelete",              || cpp::NewDelete                   ).thin().fat().print();
    #[cfg(cpp98)]               Test::new("NewDeleteArray",         || cpp::NewDeleteArray              ).thin().fat().print();
    #[cfg(cpp17)]               Test::new("NewDeleteAligned",       || cpp::NewDeleteAligned            )       .fat().print();
    #[cfg(cpp17)]               Test::new("NewDeleteArrayAligned",  || cpp::NewDeleteArrayAligned       )       .fat().print();
    #[cfg(cpp98)]               Test::new("StdAllocator<char>",     || cpp::StdAllocator::<char>::new() )       .fat().print();
    #[cfg(all(windows, feature = "win32"))] {
        println!();
        println!("win32:");
        Test::new("ProcessHeap",        || win32::ProcessHeap               ).thin().fat().print();
        Test::new("Global",             || win32::Global                    ).thin().fat().print();
        Test::new("Local",              || win32::Local                     ).thin().fat().print();
        Test::new("CryptMem",           || win32::CryptMem                  ).thin().fat().print();
        Test::new("CoTaskMem",          || win32::CoTaskMem                 ).thin().fat().print();
        Test::new("VirtualCommit",      || win32::VirtualCommit             ).thin().fat().print();
    }
}
