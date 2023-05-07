extern crate std;

use std::ffi::c_char as char;
use std::num::NonZeroUsize;

use ialloc::*;
use ialloc::allocator::*;



struct Test<A> {
    pub name:   &'static str,
    pub create: fn()->A,
    pub thin:   Option<AlignmentRange>,
    pub nzst:   Option<AlignmentRange>,
}

#[derive(Clone, Copy, Debug)] struct AlignmentRange {
    pub min: Alignment,
    pub max: Alignment,
}

impl<A> Test<A> {
    pub fn new(name: &'static str, create: fn()->A) -> Self {
        Self { name, create, thin: None, nzst: None }
    }

    pub fn thin(&mut self) -> &mut Self where A : thin::Alloc + thin::Free {
        let mut thin = AlignmentRange { min: Alignment::MAX, max: Alignment::MAX };
        for (dst, min_size) in [(&mut thin.min, 1), (&mut thin.max, 4096)] {
            for _ in 0 .. 100 {
                let alloc = (self.create)();
                let addrs = [(); 4096].into_iter().enumerate().map(|(i, _)| alloc.alloc_uninit(NonZeroUsize::new(i%16+min_size).unwrap()).unwrap()).collect::<Vec<_>>();
                let addrbits = addrs[..].iter().copied().map(|addr| addr.as_ptr() as usize).reduce(|x,y| x|y).unwrap();
                addrs.iter().copied().for_each(|addr| unsafe { alloc.free(addr) });
                let align = Alignment::new(1 << addrbits.trailing_zeros()).unwrap();
                *dst = align.min(*dst);
            }
        }
        self.thin = Some(thin);
        self
    }

    pub fn nzst(&mut self) -> &mut Self where A : nzst::Alloc + nzst::Free {
        let mut nzst = AlignmentRange { min: Alignment::MAX, max: ALIGN_1 };

        for _ in 0 .. 100 {
            let layout = LayoutNZ::new::<u8>().unwrap(); // 1B
            let alloc = (self.create)();
            let addrs = [(); 4096].map(|_| alloc.alloc_uninit(layout).unwrap());
            let addrbits = addrs[..].iter().copied().map(|addr| addr.as_ptr() as usize).reduce(|x,y| x|y).unwrap();
            addrs.iter().copied().for_each(|addr| unsafe { alloc.free(addr, layout) });
            let align = Alignment::new(1 << addrbits.trailing_zeros()).unwrap();
            nzst.min = align.min(nzst.min);
        }

        nzst.max = nzst.min;
        let alloc = (self.create)();
        while let Some(next) = nzst.max.as_usize().checked_shl(1) {
            let Some(align) = Alignment::new(next) else { break };
            let Some(size) = NonZeroUsize::new(next) else { break };
            let Ok(layout) = LayoutNZ::from_size_align(size, align) else { break };
            let Ok(addr) = alloc.alloc_uninit(layout) else { break };
            nzst.max = align;
            unsafe { alloc.free(addr, layout) };
        }

        self.nzst = Some(nzst);
        self
    }

    pub fn print(&self) {
        let name = self.name;
        let thin = self.thin.map_or_else(|| format!(""), |t| if t.min == t.max { format!("{:?}", t.min) } else { format!("{:?} ..= {:?}", t.min, t.max) });
        let nzst = self.nzst.map_or_else(|| format!(""), |t| if t.min == t.max { format!("{:?}", t.min) } else { format!("{:?} ..= {:?}", t.min, t.max) });
        println!("{name: <20}{thin: <20}{nzst: <20}");
    }
}

fn main() {
    println!("{: <20}{: <20}{: <20}", "",          "thin::Alloc",   "nzst::Alloc",  );
    println!("{: <20}{: <20}{: <20}", "Allocator", "Alignment",     "Alignment",    );
    println!("{:=<60}", "");
    #[cfg(feature = "alloc")]   Test::new("Global",                 || alloc::Global                    )       .nzst().print();
    #[cfg(c89)]                 Test::new("Malloc",                 || c::Malloc                        ).thin().nzst().print();
    #[cfg(c89)]                 Test::new("AlignedMalloc",          || c::AlignedMalloc                 )       .nzst().print();
    #[cfg(cpp98)]               Test::new("NewDelete",              || cpp::NewDelete                   ).thin().nzst().print();
    #[cfg(cpp98)]               Test::new("NewDeleteArray",         || cpp::NewDeleteArray              ).thin().nzst().print();
    #[cfg(cpp98)]               Test::new("StdAllocator<char>",     || cpp::StdAllocator::<char>::new() )       .nzst().print();
    #[cfg(all(windows, feature = "win32"))] {
        println!();
        println!("win32:");
        Test::new("ProcessHeap",        || win32::ProcessHeap               ).thin().nzst().print();
        Test::new("Global",             || win32::Global                    ).thin().nzst().print();
        Test::new("Local",              || win32::Local                     ).thin().nzst().print();
        Test::new("CryptMem",           || win32::CryptMem                  ).thin().nzst().print();
        Test::new("CoTaskMem",          || win32::CoTaskMem                 ).thin().nzst().print();
    }
}
