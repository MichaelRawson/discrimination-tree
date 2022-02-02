use crate::*;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct S(&'static str, usize);

impl fmt::Debug for S {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0, self.0)
    }
}

impl Symbol for S {
    fn arity(&self) -> usize {
        self.1
    }
}

const A: S = S("a", 0);
const B: S = S("b", 0);
const C: S = S("c", 0);
const F: S = S("f", 2);
const G: S = S("g", 2);

fn build_tree() -> DiscriminationTree<S, usize> {
    let mut tree = DiscriminationTree::default();
    let zero = || 0;

    // f(g(a, *), c) -> 1
    *tree.get_or_insert_with(
        [Some(F), Some(G), Some(A), None, Some(C)],
        zero,
    ) = 1;
    // f(g(*, b), *) -> 2
    *tree.get_or_insert_with([Some(F), Some(G), None, Some(B), None], zero) =
        2;
    // f(g(a, b), a) -> 3
    *tree.get_or_insert_with(
        [Some(F), Some(G), Some(A), Some(B), Some(A)],
        zero,
    ) = 3;
    // f(g(*, c), b) -> 4
    *tree.get_or_insert_with(
        [Some(F), Some(G), None, Some(C), Some(B)],
        zero,
    ) = 4;
    // f(*, *) -> 5
    *tree.get_or_insert_with([Some(F), None, None], zero) = 5;
    // f(g(b, c), *) -> 6
    *tree.get_or_insert_with(
        vec![Some(F), Some(G), Some(B), Some(C), None],
        zero,
    ) = 6;

    tree
}

#[test]
fn exact() {
    let tree = build_tree();
    assert!(tree
        .query([Some(F), Some(G), Some(A), None, Some(C)], false, false)
        .eq(&[1]));
}

#[test]
fn generalisation() {
    let tree = build_tree();
    assert!(tree
        .query([Some(F), Some(G), Some(A), Some(C), Some(B)], true, false)
        .eq(&[5, 4]));
}

#[test]
fn instantiation() {
    let tree = build_tree();
    assert!(tree
        .query([Some(F), None, None], false, true)
        .eq(&[5, 4, 2, 6, 1, 3]));
}

#[test]
fn unification() {
    let tree = build_tree();
    assert!(tree
        .query([Some(F), Some(G), None, Some(B), None], true, true)
        .eq(&[5, 1, 3, 2]));
}
