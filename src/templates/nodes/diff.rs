use std::iter::zip;

use crate::display::Style;
use crate::templates::WidgetLookup;
use crate::widgets::{Attributes, NodeId, WidgetContainer};

use super::{Kind, Node};

// -----------------------------------------------------------------------------
//     - Changes -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct Move {
    pub(crate) id: NodeId,
    pub(crate) new_parent: NodeId,
    pub(crate) old_parent: NodeId,
}

#[derive(Debug)]
pub(crate) struct Remove {
    pub(crate) parent: NodeId,
    pub(crate) node: Node,
}

#[derive(Debug)]
pub(crate) struct Insert {
    pub(crate) id: NodeId,
    pub(crate) parent: NodeId,
}

#[derive(Debug)]
pub(crate) struct Diff {
    pub id: NodeId,
    pub attributes: Attributes,
    pub span_diff: Vec<SpanDiff>,
}

impl Diff {
    fn is_change(&self) -> bool {
        self.attributes.is_empty() && self.span_diff.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct SpanDiff {
    index: usize,
    text: Option<String>,
    style: Option<Style>,
}

impl SpanDiff {
    pub(crate) fn new(index: usize, new_text: &str, old_text: &str, attribs: Attributes) -> Option<Self> {
        let text = if new_text != old_text { Some(new_text.to_string()) } else { None };

        let style = if !attribs.is_empty() { Some(attribs.style()) } else { None };

        if text.is_none() && style.is_none() {
            return None;
        }

        let inst = Self { index, text, style };

        Some(inst)
    }
}

/// A set of changes produced by comparing two node trees
#[derive(Debug)]
pub struct Changes {
    pub(crate) insertions: Vec<Insert>,
    pub(crate) changes: Vec<Diff>,
    pub(crate) removals: Vec<Remove>,
    pub(crate) moves: Vec<Move>,
    pub(crate) new_root: Option<NodeId>,
}

impl Changes {
    /// returns true is there are no inserts, changes, no removals and no moves
    pub fn is_empty(&self) -> bool {
        self.insertions.is_empty() && self.changes.is_empty() && self.removals.is_empty() && self.moves.is_empty()
    }

    /// Apply the changes to a `WidgetContainer`
    pub fn apply(mut self, root: &mut WidgetContainer, widget_lookup: &WidgetLookup, new_nodes: &[Node]) {
        self.finalize();

        // New root?
        if let Some(id) = self.new_root.take() {
            let node = new_nodes[0].by_id(&id).unwrap();
            let new_root = widget_lookup.make(node).unwrap();
            *root = new_root;
            return;
        }

        // Insert nodes
        for insertion in self.insertions {
            let id = insertion.id;
            let parent = insertion.parent;

            // Create node
            let node = new_nodes[0].by_id(&id).unwrap();
            let widget = widget_lookup.make(node).unwrap();

            // Find widget by parent if the widget has a parent.
            // If it doesn't it's assumed to be the root widget
            let _ = root.by_id(&parent).map(|w| w.add_child(widget));
        }

        // Changes
        for change in self.changes {
            let id = change.id;
            if let Some(node) = root.by_id(&id) {
                // Update nodes
                if !change.attributes.is_empty() {
                    node.update(change.attributes);
                }

                if !change.span_diff.is_empty() {
                    let text_widget = node.to::<crate::widgets::Text>();
                    for diff in change.span_diff {
                        text_widget.update_span(diff.index, diff.text, diff.style);
                    }
                }
            }
        }

        // Remove nodes
        for removal in self.removals {
            if let Some(parent) = root.by_id(&removal.parent) {
                parent.remove_child(&removal.node.id);
            }
        }

        // New widgets
        for m in self.moves {
            if let Some(widget) = root.by_id(&m.old_parent).and_then(|w| w.remove_child(&m.id)) {
                if let Some(parent) = root.by_id(&m.new_parent) {
                    parent.add_child(widget);
                }
            }
        }
    }

    fn new() -> Self {
        Self { insertions: Vec::new(), removals: Vec::new(), moves: Vec::new(), changes: Vec::new(), new_root: None }
    }

    fn merge(&mut self, changes: Changes) {
        self.insertions.extend(changes.insertions);
        self.removals.extend(changes.removals);
        self.moves.extend(changes.moves);
        self.changes.extend(changes.changes);
    }

    fn changed(&mut self, diff: Diff) {
        if !diff.is_change() {
            self.changes.push(diff);
        }
    }

    fn finalize(&mut self) {
        // Upgrade all insertions to moves
        // if the id exist in removals as well
        //
        // Do this until `Vec::drain_filter` is available on stable
        let mut i = 0;
        while i < self.insertions.len() {
            let id = &self.insertions[i].id;
            if let Some(pos) = self.removals.iter().position(|removal| removal.node.id.eq(id)) {
                // Upgrade insertion to move
                let removal = self.removals.remove(pos);
                let val = self.insertions.remove(i);
                self.moved(val.id, val.parent, removal.parent);
            } else {
                i += 1;
            }
        }
    }

    fn inserted(&mut self, id: NodeId, parent: NodeId) {
        self.insertions.push(Insert { id, parent });
    }

    fn removed(&mut self, parent: NodeId, node: Node) {
        self.removals.push(Remove { parent, node });
    }

    fn moved(&mut self, id: NodeId, new_parent: NodeId, old_parent: NodeId) {
        self.moves.push(Move { id, new_parent, old_parent });
    }
}

/// Create changes between two nodes, generally between a past and present node
pub fn diff(new: &Node, mut old: Node) -> Changes {
    let mut changeset = Changes::new();

    if new.id == old.id {
        let diff_attribs = new.attributes.diff(&old.attributes);
        let mut diff = Diff { id: new.id.clone(), attributes: diff_attribs, span_diff: vec![] };

        // Text diff
        if let ("text", "text") = (new.ident(), old.ident()) {
            for (idx, (new_span, old_span)) in std::iter::zip(&new.children, old.children).enumerate() {
                if let (Kind::Span(ref new_text), Kind::Span(old_text)) = (&new_span.kind, old_span.kind) {
                    let span_attribs = new_span.attributes.diff(&old_span.attributes);
                    if old_text.ne(new_text) {
                        if let Some(span_diff) = SpanDiff::new(idx, new_text, &old_text, span_attribs) {
                            diff.span_diff.push(span_diff);
                        }
                    }
                }
            }

            changeset.changed(diff);
            return changeset;
        }
        changeset.changed(diff);
    } else {
        changeset.new_root = Some(new.id());
    }

    let len = new.children.len().min(old.children.len());

    for (new_child, old_child) in zip(&new.children, old.children.drain(..len)) {
        if old_child.id == new_child.id {
            let changes = diff(new_child, old_child);
            changeset.merge(changes);
        } else {
            changeset.inserted(new_child.id.clone(), new.id.clone());
            changeset.removed(old.id.clone(), old_child);
        }
    }

    // removals
    old.children.into_iter().for_each(|c| changeset.removed(old.id.clone(), c));

    // insertions
    new.children.iter().skip(len).for_each(|c| changeset.inserted(c.id.clone(), new.id.clone()));

    changeset
}
