use crate::*;
use Term::Variable as X;
use Term::*;

fn constant(c: &'static str) -> Term<&'static str> {
    Function(c, vec![])
}

fn function(
    f: &'static str,
    children: Vec<Term<&'static str>>,
) -> Term<&'static str> {
    Function(f, children)
}

fn build_index() -> Index<&'static str, u32> {
    let mut index = Index::new();

    // f(g(a, *), c)
    index.insert(
        function(
            "f",
            vec![function("g", vec![constant("a"), X]), constant("c")],
        ),
        1,
    );
    // f(g(*, b), *)
    index.insert(
        function("f", vec![function("g", vec![X, constant("b")]), X]),
        2,
    );
    // f(g(a, b), a)
    index.insert(
        function(
            "f",
            vec![
                function("g", vec![constant("a"), constant("b")]),
                constant("a"),
            ],
        ),
        3,
    );
    // f(g(*, c), b)
    index.insert(
        function(
            "f",
            vec![function("g", vec![X, constant("c")]), constant("b")],
        ),
        4,
    );
    // f(*, *)
    index.insert(function("f", vec![X, X]), 5);
    // f(g(b, c), *)
    index.insert(
        function(
            "f",
            vec![function("g", vec![constant("b"), constant("c")]), X],
        ),
        6,
    );

    index
}

#[test]
fn insertion() {
    let index = build_index();
    assert_eq!(index.symbols.len(), 5);
    assert_eq!(index.nodes.len(), 18);
    assert_eq!(index.connections.len(), 11);
}

#[test]
fn generalisation() {
    let index = build_index();
    let query = function(
        "f",
        vec![
            function("g", vec![constant("a"), constant("c")]),
            constant("b"),
        ],
    );
    let mut unifiers = index.possible_unifiers(&query);
    assert_eq!(unifiers.next(), Some(&4));
    assert_eq!(unifiers.next(), Some(&5));
    assert_eq!(unifiers.next(), None);
}

#[test]
fn unification() {
    let index = build_index();
    let query = function(
        "f",
        vec![
            function("g", vec![constant("b"), X]),
            constant("a"),
        ],
    );
    let mut unifiers = index.possible_unifiers(&query);
    assert_eq!(unifiers.next(), Some(&6));
    assert_eq!(unifiers.next(), Some(&2));
    assert_eq!(unifiers.next(), Some(&5));
    assert_eq!(unifiers.next(), None);
}
