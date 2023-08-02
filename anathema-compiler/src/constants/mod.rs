use anathema_values::{Path, Slab, PathId};
use anathema_widget_core::{TextPath, Value};
pub(crate) use storage::Storage;

use self::paths::Paths;
use self::strings::Strings;
use self::texts::Texts;
use self::values::Values;

pub use self::strings::StringId;
pub use self::values::ValueId;
pub use self::texts::TextId;

mod paths;
mod storage;
mod strings;
mod texts;
mod values;

// -----------------------------------------------------------------------------
//   - Constants -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Constants {
    strings: Strings,
    texts: Texts,
    values: Values,
    paths: Paths,
}

impl Constants {
    pub fn new() -> Self {
        Self {
            strings: Strings::empty(),
            texts: Texts::empty(),
            values: Values::empty(),
            paths: Paths::empty(),
        }
    }

    pub(crate) fn store_string(&mut self, string: impl Into<String>) -> StringId {
        self.strings.push(string.into())
    }

    pub(crate) fn store_text(&mut self, text: TextPath) -> TextId {
        self.texts.push(text)
    }

    pub fn paths(&self) -> impl Iterator<Item = &Path> + '_ {
        self.paths.iter()
    }

    pub fn store_value(&mut self, value: Value) -> ValueId {
        self.values.push(value)
    }

    pub fn store_path(&mut self, path: Path) -> PathId {
        self.paths.push(path)
    }

    pub fn lookup_string(&self, index: StringId) -> Option<&str> {
        self.strings.get(index).map(String::as_str)
    }

    pub fn lookup_text(&self, index: TextId) -> Option<&TextPath> {
        self.texts.get(index)
    }

    pub fn lookup_value(&self, index: ValueId) -> Option<&Value> {
        self.values.get(index)
    }

    pub fn lookup_path(&self, path_id: PathId) -> Option<&Path> {
        self.paths.get(path_id)
    }
}
