use std::borrow::Cow;

use anathema_values::Value;

use crate::template::Template;

pub enum Expression<'parent> {
    Node(&'parent Template),
    View(Cow<'parent, str>),
    For {
        body: &'parent [Template],
        binding: &'parent str,
        collection: &'parent [Value],
    },
    Block(&'parent [Template]),
}

impl<'parent> Expression<'parent> {
    pub fn for_loop(
        body: &'parent [Template],
        binding: &'parent str,
        collection: &'parent [Value],
    ) -> Self {
        Self::For {
            body,
            binding,
            collection,
        }
    }
}
