use im::{ordmap::DiffItem, OrdMap};

use crate::expert::{Anchor, Engine};

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
        let mut engine = crate::singlethread::Engine::new();
        let mut dict = OrdMap::new();
        let a = crate::expert::Var::new(dict.clone());
        let b = a.watch().inner_filter(|_, n| *n > 10);
        let b_out = engine.get(&b);
        assert_eq!(0, b_out.len());

        dict.insert("a".to_string(), 1);
        dict.insert("b".to_string(), 23);
        dict.insert("c".to_string(), 5);
        dict.insert("d".to_string(), 24);
        a.set(dict.clone());
        let b_out = engine.get(&b);
        assert_eq!(2, b_out.len());
        assert_eq!(Some(&23), b_out.get("b"));
        assert_eq!(Some(&24), b_out.get("d"));

        dict.insert("a".to_string(), 25);
        dict.insert("b".to_string(), 5);
        dict.remove("d");
        dict.insert("e".to_string(), 50);
        a.set(dict.clone());
        let b_out = engine.get(&b);
        assert_eq!(2, b_out.len());
        assert_eq!(Some(&25), b_out.get("a"));
        assert_eq!(Some(&50), b_out.get("e"));
    }

    #[test]
    fn test_map() {
        let mut engine = crate::singlethread::Engine::new();
        let mut dict = OrdMap::new();
        let a = crate::expert::Var::new(dict.clone());
        let b = a.watch().inner_map(|_, n| *n + 1);
        let b_out = engine.get(&b);
        assert_eq!(0, b_out.len());

        dict.insert("a".to_string(), 1);
        dict.insert("b".to_string(), 2);
        dict.insert("c".to_string(), 3);
        dict.insert("d".to_string(), 4);
        a.set(dict.clone());
        let b_out = engine.get(&b);
        assert_eq!(4, b_out.len());
        assert_eq!(Some(&2), b_out.get("a"));
        assert_eq!(Some(&3), b_out.get("b"));
        assert_eq!(Some(&4), b_out.get("c"));
        assert_eq!(Some(&5), b_out.get("d"));

        dict.insert("a".to_string(), 10);
        dict.insert("b".to_string(), 11);
        dict.remove("d");
        dict.insert("e".to_string(), 12);
        a.set(dict.clone());
        let b_out = engine.get(&b);
        assert_eq!(4, b_out.len());
        assert_eq!(Some(&11), b_out.get("a"));
        assert_eq!(Some(&12), b_out.get("b"));
        assert_eq!(Some(&4), b_out.get("c"));
        assert_eq!(Some(&13), b_out.get("e"));
    }
}
