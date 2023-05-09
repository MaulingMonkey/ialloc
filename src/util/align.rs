use crate::Alignment;

#[track_caller] pub(crate) const fn constant(align: usize) -> Alignment { match Alignment::new(align) { Some(a) => a, None => panic!("util::align::constant(align): invalid constant") } }
