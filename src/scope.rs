use std::sync::Arc;

use crate::{named::ProgExt, ProgNode};

/// A global scope is a stack of scopes.
/// Each scope is a vector of variables.
/// The latest scope is the last vector in the stack.
///
/// Our simplicity translation looks at the index
/// of the variable from the end of stack to figure it's
/// position in the environment.
#[derive(Debug)]
pub struct GlobalScope {
    variables: Vec<Vec<Variable>>,
    witnesses: Vec<Vec<String>>,
}

impl Default for GlobalScope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum Variable {
    /// Single variable. let a = [e]. Constructed by a single assignment.
    Single(Arc<str>),
    /// Tuple variable. let (a, b) = [e]. Constructed by a tuple assignment.
    Tuple(Arc<str>, Arc<str>),
}

impl Variable {
    fn contains(&self, key: &str) -> bool {
        match self {
            Variable::Single(s) => s.as_ref() == key,
            Variable::Tuple(s1, s2) => s1.as_ref() == key || s2.as_ref() == key,
        }
    }
}

impl GlobalScope {
    /// Creates a new [`GlobalScope`].
    pub fn new() -> Self {
        GlobalScope {
            variables: vec![Vec::new()],
            witnesses: vec![Vec::new()],
        }
    }

    /// Pushes a new scope to the stack.
    pub fn push_scope(&mut self) {
        self.variables.push(Vec::new());
        self.witnesses.push(Vec::new());
    }

    /// Pops the latest scope from the stack.
    ///
    /// # Panics
    ///
    /// Panics if the stack is empty.
    pub fn pop_scope(&mut self) {
        self.variables.pop().expect("Popping scope zero");
        self.witnesses.pop().expect("Popping scope zero");
    }

    /// Pushes a new variable to the latest scope.
    pub fn insert(&mut self, key: Variable) {
        self.variables.last_mut().unwrap().push(key);
    }

    /// Pushes a new witness to the latest scope.
    pub fn insert_witness(&mut self, key: String) {
        self.witnesses.last_mut().unwrap().push(key);
    }

    /// Fetches the [`ProgNode`] for a variable.
    /// The [`ProgNode`] is a sequence of `take` and `drop` nodes
    /// that fetches the variable from the environment.
    /// The [`ProgNode`] is constructed by looking at the index
    /// of the variable from the end of stack.
    ///
    /// # Panics
    ///
    /// Panics if the variable is not found.
    pub fn get(&self, key: &str) -> ProgNode {
        // search in the vector of vectors from the end
        let mut pos = 0;
        let mut var = None;
        for v in self.variables.iter().rev() {
            if let Some(idx) = v.iter().rev().position(|var_name| var_name.contains(key)) {
                pos += idx;
                var = Some(&v[v.len() - 1 - idx]);
                break;
            } else {
                pos += v.len();
            }
        }
        match var {
            Some(v) => {
                let mut child = ProgNode::iden();
                child = match v {
                    Variable::Single(_s) => child,
                    Variable::Tuple(s1, s2) => {
                        if s1.as_ref() == key {
                            ProgNode::take(child)
                        } else if s2.as_ref() == key {
                            child = ProgNode::drop_(child);
                            child
                        } else {
                            panic!("Variable {} not found", key);
                        }
                    }
                };
                child = ProgNode::take(child);
                for _ in 0..pos {
                    child = ProgNode::drop_(child);
                }
                child
            }
            None => panic!("Variable {} not found", key),
        }
    }
}
