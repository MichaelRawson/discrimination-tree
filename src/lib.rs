#[cfg(test)]
mod tests;
mod util;

use crate::util::SortedMap;

fn report_bad_state() -> ! {
    panic!(
        "\
bad state detected - this could mean:
    1. inserted keys are not proper traversals of well-formed terms, or
    2. identical symbols have been used with different arities"
    );
}

pub trait Symbol: Ord {
    fn arity(&self) -> usize;
}

fn find_next_term<S: Symbol>(key: &[Option<S>], mut index: usize) -> usize {
    let mut remaining = 1;
    while remaining > 0 {
        remaining +=
            key[index].as_ref().map(|s| s.arity()).unwrap_or_default();
        remaining -= 1;
        index += 1;
        if index > key.len() {
            report_bad_state()
        }
    }
    index
}

#[derive(Debug, Clone)]
struct Leaf<T> {
    data: T,
}

#[derive(Debug, Clone)]
struct Branch<S, T> {
    symbols: SortedMap<S, Node<S, T>>,
    variable: Option<Box<Node<S, T>>>,
}

impl<S, T> Default for Branch<S, T> {
    fn default() -> Self {
        Self {
            symbols: SortedMap::default(),
            variable: None,
        }
    }
}

impl<S: Ord, T> Branch<S, T> {
    pub(crate) fn get(&self, item: &Option<S>) -> Option<&Node<S, T>> {
        if let Some(symbol) = item {
            self.symbols.get(symbol)
        } else {
            self.variable.as_deref()
        }
    }

    pub(crate) fn get_or_insert_empty_branch(
        &mut self,
        item: Option<S>,
    ) -> (bool, &mut Node<S, T>) {
        let mut inserted = false;
        let mut empty_branch = || {
            inserted = true;
            Node::Branch(Branch::default())
        };
        let node = if let Some(symbol) = item {
            self.symbols.get_or_insert_with(symbol, empty_branch)
        } else {
            self.variable
                .get_or_insert_with(|| Box::new(empty_branch()))
        };
        (inserted, node)
    }
}

#[derive(Debug, Clone)]
enum Node<S, T> {
    Leaf(Leaf<T>),
    Branch(Branch<S, T>),
}

struct Results<'a, S, T> {
    found: Vec<&'a T>,
    todo: Vec<(&'a Branch<S, T>, usize)>,
    skip: Vec<(&'a Branch<S, T>, usize, usize)>,
    generalise: bool,
    instantiate: bool,
}

impl<'a, S: Symbol, T> Results<'a, S, T> {
    fn add_todo(
        &mut self,
        key: &[Option<S>],
        node: &'a Node<S, T>,
        index: usize,
    ) {
        match node {
            Node::Leaf(leaf) => {
                if index != key.len() {
                    report_bad_state()
                }
                self.found.push(&leaf.data);
            }
            Node::Branch(branch) => self.todo.push((branch, index)),
        }
    }

    fn add_skip(
        &mut self,
        key: &[Option<S>],
        node: &'a Node<S, T>,
        index: usize,
        remaining: usize,
    ) {
        if remaining == 0 {
            self.add_todo(key, node, index);
        } else if let Node::Branch(branch) = node {
            self.skip.push((branch, index, remaining));
        } else {
            report_bad_state()
        }
    }

    fn do_skip_symbols(
        &mut self,
        key: &[Option<S>],
        branch: &'a Branch<S, T>,
        index: usize,
        remaining: usize,
    ) {
        let remaining = remaining - 1;
        for (symbol, node) in branch.symbols.iter() {
            let remaining = remaining + symbol.arity();
            self.add_skip(key, node, index, remaining);
        }
    }

    fn do_skip(
        &mut self,
        key: &[Option<S>],
        branch: &'a Branch<S, T>,
        index: usize,
        remaining: usize,
    ) {
        self.do_skip_symbols(key, branch, index, remaining);
        if let Some(variable) = branch.variable.as_deref() {
            self.add_skip(key, variable, index, remaining - 1);
        }
    }

    fn do_todo(
        &mut self,
        key: &[Option<S>],
        branch: &'a Branch<S, T>,
        index: usize,
    ) {
        if index >= key.len() {
            report_bad_state()
        }
        let head = &key[index];
        // exact matches: f = f, * = *
        if let Some(node) = branch.get(head) {
            self.add_todo(key, node, index + 1);
        }
        // generalisations
        if self.generalise && head.is_some() {
            if let Some(node) = branch.variable.as_deref() {
                self.add_todo(key, node, find_next_term(key, index));
            }
        }
        // instantiations
        if self.instantiate && head.is_none() {
            self.do_skip_symbols(key, branch, index + 1, 1);
        }
    }

    fn next(&mut self, key: &[Option<S>]) -> Option<&'a T> {
        loop {
            if let Some(found) = self.found.pop() {
                return Some(found);
            }
            if let Some((branch, index)) = self.todo.pop() {
                self.do_todo(key, branch, index);
            } else if let Some((branch, index, remaining)) = self.skip.pop() {
                self.do_skip(key, branch, index, remaining);
            } else {
                return None;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscriminationTree<S, T> {
    root: Branch<S, T>,
}

impl<S, T> Default for DiscriminationTree<S, T> {
    fn default() -> Self {
        Self {
            root: Branch::default(),
        }
    }
}

impl<S: Symbol, T> DiscriminationTree<S, T> {
    pub fn get_or_insert_with<
        I: IntoIterator<Item = Option<S>>,
        F: FnOnce() -> T,
    >(
        &mut self,
        key: I,
        insert: F,
    ) -> &mut T {
        let mut current = &mut self.root;
        let mut remaining = 1;
        for item in key {
            remaining -= 1;
            remaining += item.as_ref().map(|s| s.arity()).unwrap_or_default();

            let (inserted, node) = current.get_or_insert_empty_branch(item);
            if remaining == 0 {
                if inserted {
                    *node = Node::Leaf(Leaf { data: insert() });
                    if let Node::Leaf(leaf) = node {
                        return &mut leaf.data;
                    } else {
                        unreachable!();
                    }
                } else if let Node::Leaf(leaf) = node {
                    return &mut leaf.data;
                } else {
                    report_bad_state()
                }
            }
            if let Node::Branch(branch) = node {
                current = branch;
            } else {
                report_bad_state()
            }
        }
        report_bad_state()
    }

    pub fn query<I: IntoIterator<Item = Option<S>>>(
        &self,
        key: I,
        generalise: bool,
        instantiate: bool,
    ) -> impl Iterator<Item = &T> {
        let key = key.into_iter().collect::<Vec<_>>();
        let mut results = Results {
            found: vec![],
            todo: vec![(&self.root, 0)],
            skip: vec![],
            generalise,
            instantiate,
        };
        std::iter::from_fn(move || results.next(&key))
    }
}
