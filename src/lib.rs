mod arena;
#[cfg(test)]
mod tests;

use arena::{Arena, Id};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SymbolId(u32);

#[derive(PartialEq, Eq, Hash)]
struct Arity(u32);

struct Leaf<T> {
    stored: Vec<T>,
}

impl<T> Leaf<T> {
    fn new() -> Self {
        let stored = vec![];
        Leaf { stored }
    }
}

struct Branch<T> {
    variable_child: Option<Id<Node<T>>>,
    jump_list: Vec<Id<Node<T>>>,
}

impl<T> Branch<T> {
    fn new() -> Self {
        let variable_child = None;
        let jump_list = vec![];
        Branch {
            variable_child,
            jump_list,
        }
    }
}

enum Node<T> {
    Leaf(Leaf<T>),
    Branch(Branch<T>),
}

pub enum Term<Symbol> {
    Variable,
    Function(Symbol, Vec<Self>),
}

type ConnectionKey<T> = (Id<Node<T>>, SymbolId, Arity);

pub struct Index<Symbol, T> {
    symbols: BTreeMap<Symbol, SymbolId>,
    nodes: Arena<Node<T>>,
    root: Id<Node<T>>,
    connections: HashMap<ConnectionKey<T>, Id<Node<T>>>,
}

impl<Symbol: Ord, T> Index<Symbol, T> {
    pub fn new() -> Self {
        let symbols = BTreeMap::new();
        let mut nodes = Arena::new();
        let root = nodes.alloc(Node::Branch(Branch::new()));
        let connections = HashMap::new();
        Index {
            symbols,
            nodes,
            root,
            connections,
        }
    }

    fn lookup_symbol(&self, symbol: &Symbol) -> Option<SymbolId> {
        self.symbols.get(symbol).cloned()
    }

    fn store_symbol(&mut self, symbol: Symbol) -> SymbolId {
        let id = SymbolId(self.symbols.len() as u32);
        *self.symbols.entry(symbol).or_insert(id)
    }

    fn get_branch(&self, node: Id<Node<T>>) -> &Branch<T> {
        match &self.nodes[node] {
            Node::Branch(branch) => branch,
            _ => unreachable!(),
        }
    }

    fn get_branch_mut(&mut self, node: Id<Node<T>>) -> &mut Branch<T> {
        match &mut self.nodes[node] {
            Node::Branch(branch) => branch,
            _ => unreachable!(),
        }
    }

    fn get_leaf(&self, node: Id<Node<T>>) -> &Leaf<T> {
        match &self.nodes[node] {
            Node::Leaf(leaf) => leaf,
            _ => unreachable!(),
        }
    }

    fn get_leaf_mut(&mut self, node: Id<Node<T>>) -> &mut Leaf<T> {
        match &mut self.nodes[node] {
            Node::Leaf(leaf) => leaf,
            _ => unreachable!(),
        }
    }

    pub fn insert(&mut self, term: Term<Symbol>, store: T) {
        let mut current = self.root;
        let mut todo = vec![term];
        let mut jump_from = vec![];

        // traverse existing index
        while let Some(top) = todo.pop() {
            let Branch { variable_child, .. } = self.get_branch(current);
            match top {
                Term::Variable => {
                    if let Some(next) = variable_child {
                        current = *next;
                    } else {
                        todo.push(Term::Variable);
                        break;
                    }
                }
                Term::Function(f, args) => {
                    let arity = args.len() as u32;
                    if let Some(id) = self.lookup_symbol(&f) {
                        let key = (current, id, Arity(arity));
                        if let Some(next) = self.connections.get(&key) {
                            todo.extend(args.into_iter().rev());
                            jump_from.push((current, arity));
                            current = *next;
                        } else {
                            todo.push(Term::Function(f, args));
                            break;
                        }
                    } else {
                        todo.push(Term::Function(f, args));
                        break;
                    }
                }
            }
            while let Some((from, depth)) = jump_from.pop() {
                if depth != 0 {
                    jump_from.push((from, depth - 1));
                    break;
                }
            }
        }

        //insert new nodes as required
        while let Some(top) = todo.pop() {
            let next = self.nodes.next_id();
            let Branch { variable_child, .. } = self.get_branch_mut(current);
            match top {
                Term::Variable => {
                    *variable_child = Some(next);
                }
                Term::Function(f, args) => {
                    let arity = args.len() as u32;
                    let key = (current, self.store_symbol(f), Arity(arity));
                    self.connections.insert(key, next);
                    todo.extend(args.into_iter().rev());
                    jump_from.push((current, arity));
                }
            }
            let node = if todo.is_empty() {
                Node::Leaf(Leaf::new())
            } else {
                Node::Branch(Branch::new())
            };
            self.nodes.alloc(node);
            current = next;

            while let Some((from, depth)) = jump_from.pop() {
                if depth == 0 {
                    self.get_branch_mut(from).jump_list.push(current);
                } else {
                    jump_from.push((from, depth - 1));
                    break;
                }
            }
        }

        // add stored data
        self.get_leaf_mut(current).stored.push(store);
    }

    pub fn possible_unifiers<'index, 'term>(
        &'index self,
        query: &'term Term<Symbol>,
    ) -> UnificationQueryIterator<'term, 'index, Symbol, T> {
        UnificationQueryIterator {
            index: self,
            todo: vec![ChoicePoint {
                location: self.root,
                parts: vec![query],
            }],
            current: [].iter(),
        }
    }
}

impl<Symbol: Ord, T> Default for Index<Symbol, T> {
    fn default() -> Self {
        Self::new()
    }
}

struct ChoicePoint<'term, Symbol, T> {
    location: Id<Node<T>>,
    parts: Vec<&'term Term<Symbol>>,
}

pub struct UnificationQueryIterator<'term, 'index, Symbol, T> {
    index: &'index Index<Symbol, T>,
    todo: Vec<ChoicePoint<'term, Symbol, T>>,
    current: <&'index [T] as IntoIterator>::IntoIter,
}

impl<'term, 'index, Symbol: Ord, T>
    UnificationQueryIterator<'term, 'index, Symbol, T>
{
    fn step(&mut self) {
        let mut selected = self.todo.pop().unwrap();
        // leaf node, term exhausted
        if selected.parts.is_empty() {
            self.current =
                self.index.get_leaf(selected.location).stored.iter();
            return;
        }

        let branch = self.index.get_branch(selected.location);
        let top = selected.parts.pop().unwrap();
        match top {
            Term::Function(f, args) => {
                // variable children
                if let Some(next) = branch.variable_child {
                    self.todo.push(ChoicePoint {
                        location: next,
                        parts: selected.parts.clone(),
                    });
                }
                // symbol children
                if let Some(id) = self.index.lookup_symbol(f) {
                    let key =
                        (selected.location, id, Arity(args.len() as u32));
                    if let Some(next) = self.index.connections.get(&key) {
                        match &self.index.nodes[*next] {
                            Node::Branch(_) => {
                                let mut parts = selected.parts;
                                parts.extend(args.iter().rev());
                                self.todo.push(ChoicePoint {
                                    location: *next,
                                    parts,
                                });
                            }
                            Node::Leaf(Leaf { stored }) => {
                                self.current = stored.iter();
                            }
                        }
                    }
                }
            }
            // jump over query variables
            Term::Variable => {
                for jump in &branch.jump_list {
                    self.todo.push(ChoicePoint {
                        location: *jump,
                        parts: selected.parts.clone(),
                    });
                }
            }
        }
    }
}

impl<'term, 'index, Symbol: Ord, T> Iterator
    for UnificationQueryIterator<'term, 'index, Symbol, T>
{
    type Item = &'index T;

    fn next(&mut self) -> Option<&'index T> {
        loop {
            if let Some(next) = self.current.next() {
                return Some(next);
            } else if self.todo.is_empty() {
                return None;
            }
            self.step();
        }
    }
}
