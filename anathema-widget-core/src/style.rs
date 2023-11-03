use anathema_render::{Attributes, Color, Style as RenderStyle};
use anathema_values::{Context, NodeId, Value};

#[derive(Debug)]
pub struct WidgetStyle {
    pub(crate) fg: Value<Color>,
    pub(crate) bg: Value<Color>,
    pub(crate) bold: Value<bool>,
    pub(crate) dim: Value<bool>,
    pub(crate) italic: Value<bool>,
    pub(crate) underlined: Value<bool>,
    pub(crate) crossed_out: Value<bool>,
    pub(crate) overlined: Value<bool>,
    pub(crate) inverse: Value<bool>,
}

impl WidgetStyle {
    pub fn style(&self) -> RenderStyle {
        let mut attributes: Attributes = Attributes::empty();

        if let Some(true) = self.bold.value_ref() {
            attributes |= Attributes::BOLD;
        }

        if let Some(true) = self.dim.value_ref() {
            attributes |= Attributes::DIM;
        }

        if let Some(true) = self.italic.value_ref() {
            attributes |= Attributes::ITALIC;
        }

        if let Some(true) = self.underlined.value_ref() {
            attributes |= Attributes::UNDERLINED;
        }

        if let Some(true) = self.crossed_out.value_ref() {
            attributes |= Attributes::CROSSED_OUT;
        }

        if let Some(true) = self.overlined.value_ref() {
            attributes |= Attributes::OVERLINED;
        }

        if let Some(true) = self.inverse.value_ref() {
            attributes |= Attributes::INVERSE;
        }

        RenderStyle {
            fg: self.fg.value_ref().cloned(),
            bg: self.bg.value_ref().cloned(),
            attributes,
        }
    }

    pub fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
        self.fg.resolve(context, None);
        self.bg.resolve(context, None);
        self.bold.resolve(context, None);
        self.dim.resolve(context, None);
        self.italic.resolve(context, None);
        self.underlined.resolve(context, None);
        self.crossed_out.resolve(context, None);
        self.overlined.resolve(context, None);
        self.inverse.resolve(context, None);
    }
}
