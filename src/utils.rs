use std::{error::Error, mem::transmute};

pub(crate) type DynError = Box<dyn Error>;
pub(crate) type DynResult<T> = Result<T, DynError>;

pub(crate) fn the_default<T: Default>() -> T {
    Default::default()
}

pub(crate) unsafe fn transmute_lifetime<'a, T: ?Sized>(x: &T) -> &'a T {
    unsafe { transmute(x) }
}

pub(crate) unsafe fn transmute_lifetime_mut<'a, T: ?Sized>(x: &mut T) -> &'a mut T {
    unsafe { transmute(x) }
}
