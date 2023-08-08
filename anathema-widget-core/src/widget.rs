use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use anathema_generator::{Attribute, DataCtx, FromContext, NodeId};
use anathema_render::{Color, ScreenPos, Size, Style};
use anathema_values::{BucketRef, Listen, Listeners, ValueRef};

use super::contexts::{PaintCtx, PositionCtx, Unsized, WithSize};
use super::layout::Constraints;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::factory::Factory;
use crate::notifications::X;
use crate::{Display, LocalPos, Nodes, Padding, Pos, ReadOnly, Region, TextPath, Value};

// Layout:
// 1. Receive constraints
// 2. Layout children
// 3. Get children's suggested size
// 4. Apply offset to children
// 5. Get children's computed size
// ... paint

pub trait Widget {
    /// This should only be used for debugging, as there
    /// is nothing preventing one widget from having the same `kind` as another
    fn kind(&self) -> &'static str {
        "[widget]"
    }

    // -----------------------------------------------------------------------------
    //     - Layout -
    // -----------------------------------------------------------------------------
    fn layout(&mut self, children: &mut Nodes, ctx: LayoutCtx, data: &ReadOnly<'_>) -> Result<Size>;

    /// By the time this function is called the widget container
    /// has already set the position. This is useful to correctly set the position
    /// of the children.
    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx, data: &ReadOnly<'_>);

    fn paint<'tpl>(
        &mut self,
        children: &mut Nodes,
        mut ctx: PaintCtx<'_, WithSize>,
        data: &ReadOnly<'_>,
    ) {
        for (widget, children) in children.iter_mut() {
            let ctx = ctx.sub_context(None);
            widget.paint(children, ctx, data);
        }
    }
}

pub trait AnyWidget {
    fn as_any_ref(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn layout_any(
        &mut self,
        children: &mut Nodes,
        ctx: LayoutCtx,
        data: &ReadOnly<'_>,
    ) -> Result<Size>;

    fn kind_any(&self) -> &'static str;

    fn position_any(&mut self, children: &mut Nodes, ctx: PositionCtx, data: &ReadOnly<'_>);

    fn paint_any<'gen: 'ctx, 'ctx>(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>, data: &ReadOnly<'_>);
}

impl Widget for Box<dyn AnyWidget> {
    fn kind(&self) -> &'static str {
        self.deref().kind_any()
    }

    fn layout(&mut self, children: &mut Nodes, ctx: LayoutCtx, data: &ReadOnly<'_>) -> Result<Size> {
        self.deref_mut().layout_any(children, ctx, data)
    }

    fn position(&mut self, children: &mut Nodes, ctx: PositionCtx, data: &ReadOnly<'_>) {
        self.deref_mut().position_any(children, ctx, data)
    }

    fn paint(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>, data: &ReadOnly<'_>) {
        self.deref_mut().paint_any(children, ctx, data)
    }
}

impl<T: Widget + 'static> AnyWidget for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_any(
        &mut self,
        children: &mut Nodes,
        ctx: LayoutCtx,
        data: &ReadOnly<'_>,
    ) -> Result<Size> {
        self.layout(children, ctx, data)
    }

    fn kind_any(&self) -> &'static str {
        self.kind()
    }

    fn position_any(&mut self, children: &mut Nodes, ctx: PositionCtx, data: &ReadOnly<'_>) {
        self.position(children, ctx, data)
    }

    fn paint_any<'gen: 'ctx, 'ctx>(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>, data: &ReadOnly<'_>) {
        self.paint(children, ctx, data)
    }
}

impl Widget for Box<dyn Widget> {
    fn kind(&self) -> &'static str {
        self.as_ref().kind()
    }

    fn layout(
        &mut self,
        children: &mut Nodes,
        layout: LayoutCtx,
        data: &ReadOnly<'_>,
    ) -> Result<Size> {
        self.as_mut().layout(children, layout, data)
    }

    fn position<'gen, 'ctx>(&mut self, children: &mut Nodes, ctx: PositionCtx, data: &ReadOnly<'_>) {
        self.as_mut().position(children, ctx, data)
    }

    fn paint<'gen, 'ctx>(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>, data: &ReadOnly<'_>) {
        self.as_mut().paint(children, ctx, data)
    }
}

/// The `WidgetContainer` has to go through three steps before it can be displayed:
/// * [`layout`](Self::layout)
/// * [`position`](Self::position)
/// * [`paint`](Self::paint)
pub struct WidgetContainer {
    pub(crate) background: Attribute<Value>,
    pub(crate) display: Attribute<Value>,
    pub(crate) padding: Attribute<Value>,
    pub(crate) inner: Box<dyn AnyWidget>,
    pub(crate) pos: Pos,
    size: Size,
}

impl WidgetContainer {
    pub fn to_ref<T: 'static>(&self) -> &T {
        let kind = self.inner.kind();

        match self.inner.deref().as_any_ref().downcast_ref::<T>() {
            Some(t) => t,
            None => panic!("invalid widget type, found `{kind}`"),
        }
    }

    pub fn to_mut<T: 'static>(&mut self) -> &mut T {
        let kind = self.inner.kind();

        match self.inner.deref_mut().as_any_mut().downcast_mut::<T>() {
            Some(t) => t,
            None => panic!("invalid widget type, found `{kind}`"),
        }
    }

    pub fn try_to_ref<T: 'static>(&self) -> Option<&T> {
        self.inner.deref().as_any_ref().downcast_ref::<T>()
    }

    pub fn try_to_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.deref_mut().as_any_mut().downcast_mut::<T>()
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn screen_to_local(&self, screen_pos: ScreenPos) -> Option<LocalPos> {
        let pos = self.pos;

        let res = LocalPos {
            x: screen_pos.x.checked_sub(pos.x as u16)? as usize,
            y: screen_pos.y.checked_sub(pos.y as u16)? as usize,
        };

        Some(res)
    }

    pub fn outer_size(&self) -> Size {
        self.size
    }

    pub fn inner_size(&self) -> Size {
        panic!()
        // Size::new(
        //     self.size.width - (self.padding.left + self.padding.right),
        //     self.size.height - (self.padding.top + self.padding.bottom),
        // )
    }

    pub fn region(&self) -> Region {
        Region::new(
            self.pos,
            Pos::new(
                self.pos.x + self.size.width as i32,
                self.pos.y + self.size.height as i32,
            ),
        )
    }

    pub fn kind(&self) -> &'static str {
        self.inner.kind()
    }

    pub fn layout<'parent>(
        &mut self,
        children: &mut Nodes,
        constraints: Constraints,
    ) -> Result<Size> {
        match self.display.load(panic!()) {
            Display::Exclude => self.size = Size::ZERO,
            _ => {
                let layout = LayoutCtx::new(&self.templates, constraints, self.padding);
                let size = self.inner.layout(children, layout)?;

                // TODO: we should compare the new size with the old size
                //       to determine if the layout needs to propagate outwards

                self.size = size;
                self.size.width += self.padding.left + self.padding.right;
                self.size.height += self.padding.top + self.padding.bottom;
            }
        }

        Ok(self.size)
    }

    pub fn position(&mut self, children: &mut Nodes, pos: Pos, data: &ReadOnly<'_>) {
        panic!()
        // self.pos = pos;

        // let pos = Pos::new(
        //     self.pos.x + self.padding.left as i32,
        //     self.pos.y + self.padding.top as i32,
        // );

        // let ctx = PositionCtx::new(pos, self.inner_size());
        // self.inner.position(children, ctx);
    }

    pub fn paint(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, Unsized>, data: &ReadOnly<'_>) {
        panic!()
        // if let Display::Hide | Display::Exclude = self.display {
        //     return;
        // }

        // // Paint the background without the padding,
        // // using the outer size and current pos.
        // let mut ctx = ctx.into_sized(self.outer_size(), self.pos);
        // self.paint_background(&mut ctx);

        // let pos = Pos::new(
        //     self.pos.x + self.padding.left as i32,
        //     self.pos.y + self.padding.top as i32,
        // );
        // ctx.update(self.inner_size(), pos);
        // self.inner.paint(children, ctx);
    }

    fn paint_background(&self, ctx: &mut PaintCtx<'_, WithSize>) -> Option<()> {
        panic!()
        // let color = self.background?;
        // let width = self.size.width;

        // let background_str = format!("{:width$}", "", width = width);
        // let mut style = Style::new();
        // style.set_bg(color);

        // for y in 0..self.size.height {
        //     let pos = LocalPos::new(0, y);
        //     ctx.print(&background_str, style, pos);
        // }

        // Some(())
    }
}

impl FromContext for WidgetContainer {
    type Ctx = WidgetMeta;
    type Err = crate::error::Error;
    type Notifier = X;
    type Value = crate::Value;

    fn from_context(ctx: DataCtx<'_, Self>) -> Result<Self> {
        let container = WidgetContainer {
            display: ctx.get("display"),
            background: ctx.get("background"),
            padding: ctx.get("padding"),
            size: Size::ZERO,
            pos: Pos::ZERO,
            inner: Factory::exec(ctx)?,
        };
        Ok(container)
    }
}

/// Meta data needed to construct a `WidgetContainer` from a `Node`
pub struct WidgetMeta {
    pub ident: String,
    pub text: Option<TextPath>,
}
