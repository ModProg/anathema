use super::Index;

// -----------------------------------------------------------------------------
//   - Entry -
// -----------------------------------------------------------------------------
enum Entry<T> {
    Occupied(T),
    Vacant(Option<Index>),
}

pub(crate) struct Slab<T> {
    inner: Vec<Entry<T>>,
    next_id: Option<Index>,
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> Slab<T> {
    pub(crate) fn empty() -> Self {
        Self {
            inner: vec![],
            next_id: None,
        }
    }

    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
            next_id: None,
        }
    }

    pub(crate) fn get(&self, index: Index) -> Option<&T> {
        let Entry::Occupied(val) = self.inner.get(index)? else {
            return None;
        };
        Some(val)
    }

    pub(crate) fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        let Entry::Occupied(val) = self.inner.get_mut(index)? else {
            return None;
        };
        Some(val)
    }

    pub(crate) fn push(&mut self, val: T) -> Index {
        match self.next_id.take() {
            Some(index) => {
                let entry = &mut self.inner[index];
                match entry {
                    Entry::Occupied(_) => {
                        unreachable!("you found a bug with Anathema, please file a bug report")
                    }
                    Entry::Vacant(next_id) => {
                        self.next_id = next_id.take();
                        std::mem::swap(entry, &mut Entry::Occupied(val));
                        index
                    }
                }
            }
            None => {
                let index = self.inner.len();
                self.inner.push(Entry::Occupied(val));
                index
            }
        }
    }

    /// Remove the entry at a given index,
    /// and increment the generation.
    pub(crate) fn remove(&mut self, index: Index) -> T {
        let Entry::Occupied(val) = &self.inner[index] else {
            panic!("removal of vacant entry")
        };

        let mut entry = Entry::Vacant(self.next_id.take());
        self.next_id = Some(index);
        std::mem::swap(&mut self.inner[index], &mut entry);

        match entry {
            Entry::Occupied(val) => val,
            Entry::Vacant(..) => unreachable!(
                "this can't happen as we make sure it's occupied when getting the generation..."
            ),
        }
    }

    #[cfg(test)]
    fn count(&self) -> usize {
        self.inner
            .iter()
            .filter(|e| match e {
                Entry::Occupied(..) => true,
                Entry::Vacant(..) => false,
            })
            .count()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_slab() -> Slab<u32> {
        let mut slab = Slab::empty();

        slab.push(5);
        slab.push(10);
        slab.push(15);

        slab
    }

    #[test]
    fn get() {
        let mut slab = Slab::empty();
        let index = slab.push(123u8);
        let val = slab.get(index).unwrap();
        assert_eq!(*val, 123);
    }

    #[test]
    fn get_mut() {
        let mut slab = Slab::empty();
        let index = slab.push(100u8);
        let val = slab.get_mut(index).unwrap();
        assert_eq!(*val, 100);
    }

    #[test]
    fn push() {
        let mut slab = get_slab();
        let next_id = slab.count();
        let index = slab.push(100);
        assert_eq!(index, next_id);
    }

    #[test]
    fn remove() {
        let mut slab = get_slab();
        assert_eq!(slab.remove(0), 5);
    }

    #[test]
    #[should_panic(expected = "removal of vacant entry")]
    fn remove_empty() {
        let mut slab = get_slab();
        slab.remove(1);
        slab.remove(1);
    }

    #[test]
    fn multiple_removes() {
        let mut slab = get_slab();
        assert_eq!(None, slab.next_id);
        slab.remove(0);
        assert_eq!(Some(0), slab.next_id);
        slab.remove(1);
        assert_eq!(Some(1), slab.next_id);
        slab.remove(2);
        assert_eq!(Some(2), slab.next_id);
        slab.push(123);
        assert_eq!(Some(1), slab.next_id);
        slab.push(456);
        assert_eq!(Some(0), slab.next_id);
        slab.push(789);
        assert_eq!(None, slab.next_id);
    }
}
