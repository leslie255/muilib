use std::marker::PhantomData;

use bumpalo::Bump;

use crate::{Axis, View};

mod container;
mod stack;

pub use container::*;
pub use stack::*;

pub struct LayoutPass<'cx> {
    bumpalo: Bump,
    _marker: PhantomData<&'cx ()>,
}

impl Default for LayoutPass<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'cx> LayoutPass<'cx> {
    pub(crate) fn new() -> Self {
        Self {
            bumpalo: Bump::new(),
            _marker: PhantomData,
        }
    }

    pub fn container<'pass, 'view, Subview>(
        &'pass self,
        subview: &'view mut Subview,
    ) -> &'pass mut Container<'view, Subview>
    where
        Subview: View<'cx>,
    {
        self.bumpalo.alloc(Container::new(subview))
    }

    pub fn stack<'pass, 'views>(
        &'pass self,
        axis: Axis,
        build: impl FnOnce(&mut StackBuilder<'pass, 'views, 'cx>),
    ) -> &'pass mut Stack<'pass, 'views, 'cx> {
        let mut builder = StackBuilder::new(&self.bumpalo, axis);
        build(&mut builder);
        self.bumpalo.alloc(builder.finish())
    }

    pub fn vstack<'pass, 'views>(
        &'pass self,
        build: impl FnOnce(&mut StackBuilder<'pass, 'views, 'cx>),
    ) -> &'pass mut Stack<'pass, 'views, 'cx> {
        self.stack(Axis::Vertical, build)
    }

    pub fn hstack<'pass, 'views>(
        &'pass self,
        build: impl FnOnce(&mut StackBuilder<'pass, 'views, 'cx>),
    ) -> &'pass mut Stack<'pass, 'views, 'cx> {
        self.stack(Axis::Horizontal, build)
    }
}
