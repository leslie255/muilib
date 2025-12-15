use std::marker::PhantomData;

use bumpalo::Bump;
use derive_more::{AsMut, AsRef, Deref, DerefMut};

use crate::{Axis, RectSize, View};

mod container;
mod stack;

pub use container::*;
pub use stack::*;

pub struct LayoutPass<'cx, UiState: 'cx> {
    bumpalo: Bump,
    _marker: PhantomData<&'cx UiState>,
}

impl<UiState> Default for LayoutPass<'_, UiState> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'cx, UiState: 'cx> LayoutPass<'cx, UiState> {
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
        Subview: View<'cx, UiState>,
    {
        self.bumpalo.alloc(Container::new(subview))
    }

    pub fn stack<'pass, 'views>(
        &'pass self,
        axis: Axis,
        build: impl FnOnce(&mut StackBuilder<'pass, 'views, 'cx, UiState>),
    ) -> &'pass mut Stack<'pass, 'views, 'cx, UiState> {
        let mut builder = StackBuilder::new(&self.bumpalo, axis);
        build(&mut builder);
        self.bumpalo.alloc(builder.finish())
    }

    pub fn vstack<'pass, 'views>(
        &'pass self,
        build: impl FnOnce(&mut StackBuilder<'pass, 'views, 'cx, UiState>),
    ) -> &'pass mut Stack<'pass, 'views, 'cx, UiState> {
        self.stack(Axis::Vertical, build)
    }

    pub fn hstack<'pass, 'views>(
        &'pass self,
        build: impl FnOnce(&mut StackBuilder<'pass, 'views, 'cx, UiState>),
    ) -> &'pass mut Stack<'pass, 'views, 'cx, UiState> {
        self.stack(Axis::Horizontal, build)
    }
}

#[derive(AsRef, AsMut, Deref, DerefMut)]
pub(crate) struct Subview<'a, 'cx, UiState> {
    pub(crate) preferred_size: RectSize<f32>,
    #[deref]
    #[deref_mut]
    #[as_ref]
    #[as_mut]
    pub(crate) view: &'a mut (dyn View<'cx, UiState> + 'a),
}
