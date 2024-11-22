use super::*;

fn to_vec<I: std::iter::Iterator>(iter: I) -> Vec<I::Item> {
    iter.collect()
}

#[test]
fn set_edge_updates_correctly() {
    let graph = Graph::new(256);

    graph.with(|guard| {
        let a = guard.insert_testing_guard();
        let b = guard.insert_testing_guard();

        assert_eq!(to_vec(a.necessary_children()), vec![]);
        assert_eq!(to_vec(a.clean_parents()), vec![]);
        assert_eq!(to_vec(b.necessary_children()), vec![]);
        assert_eq!(to_vec(b.clean_parents()), vec![]);
        assert_eq!(a.necessary_count.get(), 0);
        assert_eq!(b.necessary_count.get(), 0);

        assert!(!ensure_height_increases(a, b).unwrap());
        assert!(ensure_height_increases(a, b).unwrap());

        a.add_clean_parent(b);

        assert_eq!(to_vec(a.necessary_children()), vec![]);
        assert_eq!(to_vec(a.clean_parents()), vec![b]);
        assert_eq!(to_vec(b.necessary_children()), vec![]);
        assert_eq!(to_vec(b.clean_parents()), vec![]);
        assert_eq!(a.necessary_count.get(), 0);
        assert_eq!(b.necessary_count.get(), 0);

        assert!(ensure_height_increases(a, b).unwrap());

        b.add_necessary_child(a);

        assert_eq!(to_vec(a.necessary_children()), vec![]);
        assert_eq!(to_vec(a.clean_parents()), vec![b]);
        assert_eq!(to_vec(b.necessary_children()), vec![a]);
        assert_eq!(to_vec(b.clean_parents()), vec![]);
        assert_ne!(a.necessary_count.get(), 0);
        assert_eq!(b.necessary_count.get(), 0);

        let _ = a.drain_clean_parents();

        assert_eq!(to_vec(a.necessary_children()), vec![]);
        assert_eq!(to_vec(a.clean_parents()), vec![]);
        assert_eq!(to_vec(b.necessary_children()), vec![a]);
        assert_eq!(to_vec(b.clean_parents()), vec![]);
        assert_ne!(a.necessary_count.get(), 0);
        assert_eq!(b.necessary_count.get(), 0);

        let _ = b.drain_necessary_children();

        assert_eq!(to_vec(a.necessary_children()), vec![]);
        assert_eq!(to_vec(a.clean_parents()), vec![]);
        assert_eq!(to_vec(b.necessary_children()), vec![]);
        assert_eq!(to_vec(b.clean_parents()), vec![]);
        assert_eq!(a.necessary_count.get(), 0);
        assert_eq!(b.necessary_count.get(), 0);
    });
}

#[test]
fn height_calculated_correctly() {
    let graph = Graph::new(256);

    graph.with(|guard| {
        let a = guard.insert_testing_guard();
        let b = guard.insert_testing_guard();
        let c = guard.insert_testing_guard();

        assert_eq!(height(a), 0);
        assert_eq!(height(b), 0);
        assert_eq!(height(c), 0);

        assert!(!ensure_height_increases(b, c).unwrap());
        assert!(ensure_height_increases(b, c).unwrap());

        b.add_clean_parent(c);

        assert_eq!(height(a), 0);
        assert_eq!(height(b), 0);
        assert_eq!(height(c), 1);

        assert!(!ensure_height_increases(a, b).unwrap());
        assert!(ensure_height_increases(a, b).unwrap());

        a.add_clean_parent(b);

        assert_eq!(height(a), 0);
        assert_eq!(height(b), 1);
        assert_eq!(height(c), 2);

        let _ = a.drain_clean_parents();

        assert_eq!(height(a), 0);
        assert_eq!(height(b), 1);
        assert_eq!(height(c), 2);
    })
}

#[test]
fn cycles_cause_error() {
    let graph = Graph::new(256);

    graph.with(|guard| {
        let b = guard.insert_testing_guard();
        let c = guard.insert_testing_guard();
        ensure_height_increases(b, c).unwrap();
        b.add_clean_parent(c);
        ensure_height_increases(c, b).unwrap_err();
    })
}

#[test]
fn non_cycles_wont_cause_errors() {
    let graph = Graph::new(256);

    graph.with(|guard| {
        let a = guard.insert_testing_guard();
        let b = guard.insert_testing_guard();
        let c = guard.insert_testing_guard();
        let d = guard.insert_testing_guard();
        let e = guard.insert_testing_guard();

        ensure_height_increases(b, c).unwrap();
        b.add_clean_parent(c);
        ensure_height_increases(c, e).unwrap();
        c.add_clean_parent(e);
        ensure_height_increases(b, d).unwrap();
        b.add_clean_parent(d);
        ensure_height_increases(d, e).unwrap();
        d.add_clean_parent(e);
        ensure_height_increases(a, b).unwrap();
        a.add_clean_parent(b);
    })
}

#[test]
fn test_insert_pop() {
    let graph = Graph::new(10);

    graph.with(|guard| {
        let a = guard.insert_testing_guard();
        set_min_height(a, 0).unwrap();
        let b = guard.insert_testing_guard();
        set_min_height(b, 5).unwrap();
        let c = guard.insert_testing_guard();
        set_min_height(c, 3).unwrap();
        let d = guard.insert_testing_guard();
        set_min_height(d, 4).unwrap();
        let e = guard.insert_testing_guard();
        set_min_height(e, 1).unwrap();
        let e2 = guard.insert_testing_guard();
        set_min_height(e2, 1).unwrap();
        let e3 = guard.insert_testing_guard();
        set_min_height(e3, 1).unwrap();

        guard.queue_recalc(a);
        guard.queue_recalc(a);
        guard.queue_recalc(a);
        guard.queue_recalc(b);
        guard.queue_recalc(c);
        guard.queue_recalc(d);

        assert_eq!(guard.recalc_pop_next().map(|(_, v)| v).unwrap(), a);
        assert_eq!(guard.recalc_pop_next().map(|(_, v)| v).unwrap(), c);
        assert_eq!(guard.recalc_pop_next().map(|(_, v)| v).unwrap(), d);

        guard.queue_recalc(e);
        guard.queue_recalc(e2);
        guard.queue_recalc(e3);

        assert_eq!(guard.recalc_pop_next().map(|(_, v)| v).unwrap(), e3);
        assert_eq!(guard.recalc_pop_next().map(|(_, v)| v).unwrap(), e2);
        assert_eq!(guard.recalc_pop_next().map(|(_, v)| v).unwrap(), e);
        assert_eq!(guard.recalc_pop_next().map(|(_, v)| v).unwrap(), b);

        assert!(guard.recalc_pop_next().map(|(_, v)| v).is_none());
    })
}

#[test]
#[should_panic]
fn test_insert_above_max_height() {
    let graph = Graph::new(10);

    graph.with(|guard| {
        let a = guard.insert_testing_guard();
        set_min_height(a, 10).unwrap();
        guard.queue_recalc(a);
    })
}

#[test]
fn test_free_list() {
    use crate::expert::AnchorHandle;

    let graph = Graph::new(10);

    let a = graph.insert_testing();
    let b = graph.insert_testing();
    let c = graph.insert_testing();

    let a_token = a.token();
    let b_token = b.token();
    let c_token = c.token();

    std::mem::drop(a);
    std::mem::drop(b);
    std::mem::drop(c);

    let c = graph.insert_testing();
    let b = graph.insert_testing();
    let a = graph.insert_testing();
    let d = graph.insert_testing();

    assert_eq!(a.token(), a_token);
    assert_eq!(b.token(), b_token);
    assert_eq!(c.token(), c_token);

    let d_token = d.token();

    std::mem::drop(c);
    std::mem::drop(a);
    std::mem::drop(b);
    std::mem::drop(d);

    let d = graph.insert_testing();
    let b = graph.insert_testing();
    let a = graph.insert_testing();
    let c = graph.insert_testing();

    assert_eq!(a.token(), a_token);
    assert_eq!(b.token(), b_token);
    assert_eq!(c.token(), c_token);
    assert_eq!(d.token(), d_token);
}
