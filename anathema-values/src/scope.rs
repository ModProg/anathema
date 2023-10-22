use std::rc::Rc;

use crate::hashmap::HashMap;
use crate::{Attributes, NodeId, Path, State, Value, ValueRef};

#[derive(Debug, Clone)]
pub enum ScopeValue<'a> {
    Static(ValueRef<'a>),
    Dyn(&'a Path),
}

#[derive(Debug)]
pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    inner: HashMap<Path, ScopeValue<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'_>>) -> Self {
        Self {
            parent,
            inner: HashMap::new(),
        }
    }

    pub fn reparent(&self) -> Scope<'_> {
        Scope::new(Some(self))
    }

    pub fn scope(&mut self, path: Path, value: ScopeValue<'a>) {
        self.inner.insert(path, value);
    }

    pub fn lookup(&self, path: &Path) -> Option<ValueRef<'a>> {
        match self.inner.get(path) {
            Some(ScopeValue::Static(value)) => Some(*value),
            Some(ScopeValue::Dyn(path)) => self.lookup(path),
            None => self.parent?.lookup(path),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Context<'a: 'val, 'val> {
    pub state: &'a dyn State,
    pub scope: &'a Scope<'val>,
}

impl<'a, 'val> Context<'a, 'val> {
    pub fn new(state: &'a dyn State, scope: &'a Scope<'val>) -> Self {
        Self { state, scope }
    }

    pub fn lookup(&self, path: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'a>> {
        self.scope
            .lookup(path)
            .or_else(|| self.state.get(path, node_id))
    }

    /// Try to find the value in the current scope,
    /// if there is no value fallback to look for the value in the state.
    /// This will recursively lookup dynamic values
    pub fn get<T: ?Sized>(&self, path: &Path, node_id: Option<&NodeId>) -> Option<&'val T>
    where
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        self.lookup(path, node_id)
            .and_then(|value_ref| <&T>::try_from(value_ref).ok())
    }

    pub fn attribute<T: ?Sized>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &'val Attributes,
    ) -> Option<&'val T>
    where
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        attributes
            .get(key.as_ref())
            .and_then(|expr| expr.eval(self, node_id))
    }

    pub fn owned<T>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &'val Attributes,
    ) -> Option<T>
    where
        T: Copy,
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        attributes
            .get(key.as_ref())
            .and_then(|expr| expr.eval(self, node_id))
            .map(|val| *val)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{testing::*, ValueExpr};

    type Sub = usize;

    #[test]
    fn scope_value() {
        let mut scope = Scope::new(None);
        scope.scope(
            "value".into(),
            ScopeValue::Static(ValueRef::Str("hello world")),
        );

        let mut inner = scope.reparent();

        inner.scope(
            "value".into(),
            ScopeValue::Static(ValueRef::Str("inner hello")),
        );
        let ValueRef::Str(lhs) = inner.lookup(&"value".into()).unwrap() else {
            panic!()
        };

        assert_eq!(lhs, "inner hello");

        let ValueRef::Str(lhs) = scope.lookup(&"value".into()).unwrap() else {
            panic!()
        };
        assert_eq!(lhs, "hello world");
    }

    #[test]
    fn dynamic_attribute() {
        let mut state = TestState::new();
        let mut root = Scope::new(None);
        let ctx = Context::new(&mut state, &mut root);
        let mut attributes = Attributes::new();
        attributes.insert("name".to_string(), ValueExpr::Ident("name".into()));

        let id = Some(123.into());
        let name: &str = ctx.attribute("name", id.as_ref(), &attributes).unwrap();
        assert_eq!("Dirk Gently", name);
    }

    #[test]
    fn context_lookup() {
        let state = TestState::new();
        let scope = Scope::new(None);
        let context = Context::new(&state, &scope);

        let path = Path::from("inner").compose("name");
        let value = context.lookup(&path, None).unwrap();
        assert!(matches!(value, ValueRef::Str("Fiddle McStick")));
    }
}
