// src/tree.rs

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Shared reference to a node: multiple owners, interior mutability.
pub type NodeRef = Rc<RefCell<Node>>;

#[derive(Debug)]
pub struct Node {
    pub id: u32,
    pub title: String,
    pub done: bool,
    pub children: Vec<NodeRef>, // Child nodes of this node
}

impl Node {
    pub fn new(id: u32, title: impl Into<String>) -> NodeRef {
        Rc::new(RefCell::new(Node {
            id,
            title: title.into(),
            done: false,
            children: Vec::new(),
        }))
    }
}

pub struct Tree {
    roots: Vec<NodeRef>,
    index: HashMap<u32, NodeRef>,
    next_id: u32,
}

#[allow(dead_code)]
impl Tree {
    /// Create an empty tree
    pub fn new() -> Self {
        Tree {
            roots: Vec::new(),
            index: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a new root node. Returns its ID.
    pub fn add_root(&mut self, title: impl Into<String>) -> u32 {
        let id = self.alloc_id();
        let node = Node::new(id, title);

        self.index.insert(id, Rc::clone(&node));
        self.roots.push(node);

        id
    }

    /// Add a child under `parent_id`. Returns child ID on success
    pub fn add_child(&mut self, parent_id: u32, title: impl Into<String>) -> Option<u32> {
        let parent = self.index.get(&parent_id)?.clone();

        let id = self.alloc_id();
        let child = Node::new(id, title);

        parent.borrow_mut().children.push(child.clone());
        self.index.insert(id, child);

        Some(id)
    }

    /// Toggle the `done` flag for a node. Returns `true` if found.
    pub fn toggle(&mut self, id: u32) -> bool {
        if let Some(node) = self.index.get(&id) {
            // If the given id exists, toggle its `done` flag
            let mut n = node.borrow_mut();
            n.done = !n.done;
            true
        } else {
            false
        }
    }

    /// Get a read-only handle to a node.
    pub fn get(&self, id: u32) -> Option<NodeRef> {
        self.index.get(&id).cloned()
    }

    /// Print the tree to the given formatter
    pub fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for root in &self.roots {
            Self::fmt_node(root, 0, f)?;
        }

        Ok(())
    }

    fn fmt_node(node_ref: &NodeRef, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let node = node_ref.borrow();

        // indentation
        for _ in 0..indent {
            write!(f, "  ")?;
        }

        writeln!(
            f,
            "[{}] {} (id: {})",
            if node.done { "x" } else { " " },
            node.title,
            node.id
        )?;

        for child in &node.children {
            Self::fmt_node(child, indent + 1, f)?;
        }

        Ok(())
    }

    /// Allocate a new unique ID
    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        id
    }
}

/// For `println!("{}", tree);`
impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_pretty(f)
    }
}
