use std::collections::HashMap;
use std::sync::OnceLock;

use anathema_values::State;
use parking_lot::RwLock;

pub use self::context::FactoryContext;
use crate::error::{Error, Result};
use crate::widget::AnyWidget;

mod context;

const RESERVED_NAMES: &[&str] = &["if", "for", "else", "with"];

pub trait WidgetFactory: Send + Sync {
    fn make(&self, context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>>;
}

static FACTORIES: OnceLock<RwLock<HashMap<String, Box<dyn WidgetFactory>>>> = OnceLock::new();

pub struct Factory;

impl Factory {
    pub fn exec(ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let factories = FACTORIES.get_or_init(Default::default).read();
        let factory = factories
            .get(ctx.ident)
            .ok_or_else(|| Error::UnregisteredWidget(ctx.ident.to_string()))?;
        let widget = factory.make(ctx)?;
        Ok(Box::new(widget))
    }

    pub fn register(ident: impl Into<String>, factory: impl WidgetFactory + 'static) -> Result<()> {
        let ident = ident.into();
        if RESERVED_NAMES.contains(&ident.as_str()) {
            return Err(Error::ReservedName(ident));
        }

        let mut factories = FACTORIES.get_or_init(Default::default).write();
        if factories.contains_key(&ident) {
            return Err(Error::ExistingName(ident));
        }

        factories.insert(ident, Box::new(factory));

        Ok(())
    }
}

// // // -----------------------------------------------------------------------------
// // //     - Canvas -
// // // -----------------------------------------------------------------------------
// // fn canvas_widget<'gen, 'ctx>(
// //     node: &'gen WidgetTemplate,
// //     _: &WidgetLookup,
// // ) -> Result<WidgetContainer<'gen>> {
// //     panic!()
// //     // let attribs = &node.attributes;
// //     // let widget = Canvas::new(attribs.width(), attribs.height());
// //     // Ok(widget.into_container(node.id()))
// // }

// fn item_widget(_: ValueLookup<'_>) -> Result<Box<dyn AnyWidget>> {
//     Ok(Box::new(Item))
// }

// #[cfg(test)]
// mod test {
//     // use anathema_widgets::{fields, Attributes, BorderStyle, NodeId};

//     // use super::*;

//     // fn node_to_widget(kind: &WidgetKind, attribs: &Attributes) -> WidgetContainer {
//     //     let lookup = WidgetLookup::default();
//     //     lookup.make(kind, attribs).unwrap()
//     // }

//     // #[test]
//     // fn lookup_border() {
//     //     let mut attributes = Attributes::empty();
//     //     attributes.set(fields::MIN_WIDTH, 10u64);
//     //     attributes.set(fields::MIN_HEIGHT, 3u64);
//     //     attributes.set(fields::BORDER_STYLE, BorderStyle::Custom("01234567".into()));
//     //     let node = WidgetTemplate {
//     //         kind: TemplateKind::Node(WidgetKind::Node("border".into()), attributes),
//     //         children: vec![],
//     //         id: NodeId::empty(),
//     //     };

//     //     let mut widget = node_to_widget(&node);
//     //     let border = widget.to_mut::<Border>();
//     //     assert_eq!(Some(10), border.min_width);
//     //     assert_eq!(Some(3), border.min_height);
//     //     assert_eq!(['0', '1', '2', '3', '4', '5', '6', '7'], border.edges);
//     // }

//     // #[test]
//     // fn lookup_vstack() {
//     //     let mut attributes = Attributes::empty();
//     //     attributes.set(fields::MIN_WIDTH, 10u64);
//     //     attributes.set(fields::MIN_HEIGHT, 3u64);
//     //     let node = WidgetTemplate {
//     //         kind: TemplateKind::Node(WidgetKind::Node("vstack".into()), attributes),
//     //         children: vec![],
//     //         id: NodeId::empty(),
//     //     };

//     //     let mut widget = node_to_widget(&node);
//     //     let stack = widget.to_mut::<VStack>();
//     //     assert_eq!(Some(10), stack.min_width);
//     //     assert_eq!(Some(3), stack.min_height);
//     // }

//     // #[test]
//     // fn lookup_hstack() {
//     //     let mut attributes = Attributes::empty();
//     //     attributes.set(fields::MIN_WIDTH, 10u64);
//     //     attributes.set(fields::MIN_HEIGHT, 3u64);
//     //     let node = WidgetTemplate {
//     //         kind: TemplateKind::Node(WidgetKind::Node("hstack".into()), attributes),
//     //         children: vec![],
//     //         id: NodeId::empty(),
//     //     };

//     //     let mut widget = node_to_widget(&node);
//     //     let stack = widget.to_mut::<HStack>();
//     //     assert_eq!(Some(10), stack.min_width);
//     //     assert_eq!(Some(3), stack.min_height);
//     // }

//     // #[test]
//     // fn lookup_zstack() {
//     //     let mut attributes = Attributes::empty();
//     //     attributes.set(fields::MIN_WIDTH, 10u64);
//     //     attributes.set(fields::MIN_HEIGHT, 3u64);
//     //     let node = WidgetTemplate {
//     //         kind: TemplateKind::Node(WidgetKind::Node("zstack".into()), attributes),
//     //         children: vec![],
//     //         id: NodeId::empty(),
//     //     };

//     //     let mut widget = node_to_widget(&node);
//     //     let stack = widget.to_mut::<ZStack>();
//     //     assert_eq!(Some(10), stack.min_width);
//     //     assert_eq!(Some(3), stack.min_height);
//     // }
// }
