use im::{ordmap::DiffItem, OrdMap};

use crate::{core::Engine, Anchor};

impl<E, K, V> Anchor<OrdMap<K, V>, E>
where
    E: Engine,
    K: Ord + Clone + PartialEq + 'static,
    V: Clone + PartialEq + 'static,
{
    // TODO MERGE FN
    pub fn inner_filter(
        &self,
        mut f: impl 'static + FnMut(&K, &V) -> bool,
    ) -> Anchor<OrdMap<K, V>, E> {
        self.inner_filter_map(move |k, v| if f(k, v) { Some(v.clone()) } else { None })
    }

    pub fn inner_map<T>(&self, mut f: impl 'static + FnMut(&K, &V) -> T) -> Anchor<OrdMap<K, T>, E>
    where
        T: 'static + Clone + PartialEq,
    {
        self.inner_filter_map(move |k, v| Some(f(k, v)))
    }

    pub fn inner_filter_map<T>(
        &self,
        mut f: impl 'static + FnMut(&K, &V) -> Option<T>,
    ) -> Anchor<OrdMap<K, T>, E>
    where
        T: 'static + Clone + PartialEq,
    {
        self.inner_unordered_fold(OrdMap::new(), move |out, diff_item| {
            match diff_item {
                DiffItem::Add(k, v) => {
                    if let Some(new) = f(k, v) {
                        out.insert(k.clone(), new);
                        return true;
                    }
                }
                DiffItem::Update {
                    new: (k, v),
                    old: _,
                } => {
                    if let Some(new) = f(k, v) {
                        out.insert(k.clone(), new);
                        return true;
                    } else if out.contains_key(k) {
                        out.remove(k);
                        return true;
                    }
                }
                DiffItem::Remove(k, _v) => {
                    out.remove(k);
                    return true;
                }
            }
            false
        })
    }

    pub fn inner_unordered_fold<T>(
        &self,
        initial_state: T,
        mut f: impl 'static + for<'a> FnMut(&mut T, DiffItem<'a, K, V>) -> bool,
    ) -> Anchor<T, E>
    where
        T: 'static + PartialEq + Clone,
    {
        let mut last_observation = OrdMap::new();
        self.map_mut(initial_state, move |out, this| {
            let mut did_update = false;
            for item in last_observation.diff(this) {
                if f(out, item) {
                    did_update = true;
                }
            }
            last_observation = this.clone();
            did_update
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_filter() {
        use crate::single_threaded::{Engine, Variable};

        let mut engine = Engine::new();

        let mut a_map = OrdMap::new();
        let a = Variable::new(a_map.clone());

        let b = a.watch().inner_filter(|_, n| *n > 10);
        let b_map = engine.get(&b);

        assert_eq!(0, b_map.len());

        a_map.insert("a".to_string(), 1);
        a_map.insert("b".to_string(), 23);
        a_map.insert("c".to_string(), 5);
        a_map.insert("d".to_string(), 24);

        a.set(a_map.clone());

        let b_map = engine.get(&b);
        assert_eq!(2, b_map.len());

        assert_eq!(Some(&23), b_map.get("b"));
        assert_eq!(Some(&24), b_map.get("d"));

        a_map.insert("a".to_string(), 25);
        a_map.insert("b".to_string(), 5);
        a_map.remove("d");
        a_map.insert("e".to_string(), 50);

        a.set(a_map.clone());

        let b_map = engine.get(&b);
        assert_eq!(2, b_map.len());

        assert_eq!(Some(&25), b_map.get("a"));
        assert_eq!(Some(&50), b_map.get("e"));
    }

    #[test]
    fn test_map() {
        use crate::single_threaded::{Engine, Variable};

        let mut engine = Engine::new();

        let mut a_map = OrdMap::new();
        let a = Variable::new(a_map.clone());

        let b = a.watch().inner_map(|_, n| *n + 1);
        let b_map = engine.get(&b);

        assert_eq!(0, b_map.len());

        a_map.insert("a".to_string(), 1);
        a_map.insert("b".to_string(), 2);
        a_map.insert("c".to_string(), 3);
        a_map.insert("d".to_string(), 4);

        a.set(a_map.clone());

        let b_map = engine.get(&b);
        assert_eq!(4, b_map.len());

        assert_eq!(Some(&2), b_map.get("a"));
        assert_eq!(Some(&3), b_map.get("b"));
        assert_eq!(Some(&4), b_map.get("c"));
        assert_eq!(Some(&5), b_map.get("d"));

        a_map.insert("a".to_string(), 10);
        a_map.insert("b".to_string(), 11);
        a_map.remove("d");
        a_map.insert("e".to_string(), 12);

        a.set(a_map.clone());

        let b_map = engine.get(&b);
        assert_eq!(4, b_map.len());

        assert_eq!(Some(&11), b_map.get("a"));
        assert_eq!(Some(&12), b_map.get("b"));
        assert_eq!(Some(&4), b_map.get("c"));
        assert_eq!(Some(&13), b_map.get("e"));
    }
}
