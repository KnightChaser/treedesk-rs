// src/tree.rs

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::{Rc, Weak};

/// Shared reference to a node: multiple owners, interior mutability.
pub type NodeRef = Rc<RefCell<Node>>;

#[derive(Debug)]
pub struct Node {
    pub id: u32,
    pub title: String,
    pub done: bool,
    pub children: Vec<NodeRef>,              // Child nodes of this node
    pub parent: Option<Weak<RefCell<Node>>>, // Weak pointer to the node
}

impl Node {
    pub fn new(id: u32, title: impl Into<String>, parent: Option<Weak<RefCell<Node>>>) -> NodeRef {
        Rc::new(RefCell::new(Node {
            id,
            title: title.into(),
            done: false,
            children: Vec::new(),
            parent: parent,
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
        let node = Node::new(id, title, None);

        self.index.insert(id, Rc::clone(&node));
        self.roots.push(node);

        id
    }

    /// Add a child under `parent_id`. Returns child ID on success
    pub fn add_child(&mut self, parent_id: u32, title: impl Into<String>) -> Option<u32> {
        let parent = self.index.get(&parent_id)?.clone();

        let id = self.alloc_id();
        let parent_weak = Rc::downgrade(&parent);
        let child = Node::new(id, title, Some(parent_weak));

        parent.borrow_mut().children.push(child.clone());
        self.index.insert(id, child);

        // recompute done flags upward from parent
        Self::propagate_done_upward(&parent);

        Some(id)
    }

    /// Toggle the `done` flag for a node. Returns `true` if found.
    ///
    /// After toggling, completion status is propagated upwards:
    /// a parent becomes done if *all* its children are done.
    pub fn toggle(&mut self, id: u32) -> bool {
        if let Some(node) = self.index.get(&id).cloned() {
            {
                // If the given id exists, toggle its `done` flag
                let mut n = node.borrow_mut();
                n.done = !n.done;
            }

            Self::propagate_done_upward(&node);
            true
        } else {
            false
        }
    }

    /// Delete a node and its subtree. Returns `true` if found.
    ///
    /// - Detaches it from parent or roots
    /// - Removes it and all descendants from the index
    /// - Recomputes parent completion upwards
    pub fn delete(&mut self, id: u32) -> bool {
        let Some(node_ref) = self.index.get(&id).cloned() else {
            return false;
        };

        // 1. Detach from the parent or from roots
        let parent_weak_opt = {
            let node = node_ref.borrow();
            node.parent.clone()
        };

        if let Some(parent_weak) = parent_weak_opt {
            if let Some(parent_rc) = parent_weak.upgrade() {
                {
                    let mut parent = parent_rc.borrow_mut();
                    parent
                        .children
                        .retain(|child_ref| child_ref.borrow().id != id);
                }

                // Recompute done flags upward from parent
                Self::propagate_done_upward(&parent_rc);
            }
        } else {
            // It's a root node
            self.roots.retain(|root_ref| root_ref.borrow().id != id);
        }

        // 2. Remove from index (this node + all descendants)
        self.remove_from_index_rec(&node_ref);

        true
    }

    /// Move a node to a new parent. Returns `true` on success
    ///
    /// - Fails if `id == new_parent_id`
    /// - Fails if `new_parent` is in the subtree of `id` (would create a cycle)
    pub fn move_node(&mut self, id: u32, new_parent_id: u32) -> bool {
        if id == new_parent_id {
            return false;
        }

        let Some(node_ref) = self.index.get(&id).cloned() else {
            return false;
        };
        let Some(new_parent) = self.index.get(&new_parent_id).cloned() else {
            return false;
        };

        if Self::is_descendant(&node_ref, &new_parent) {
            return false;
        }

        // 1. Detach from old parent or roots
        let old_parent_weak_opt = {
            let node = node_ref.borrow();
            node.parent.clone()
        };

        if let Some(old_parent_weak) = old_parent_weak_opt {
            if let Some(old_parent_rc) = old_parent_weak.upgrade() {
                {
                    let mut old_parent = old_parent_rc.borrow_mut();
                    old_parent
                        .children
                        .retain(|child_ref| child_ref.borrow().id != id);
                }

                // Recompute done flags upward from old parent
                Self::propagate_done_upward(&old_parent_rc);
            }
        } else {
            // It was a root node
            self.roots.retain(|root_ref| root_ref.borrow().id != id);
        }

        // 2. Attach to new parent
        {
            let mut node_mut = node_ref.borrow_mut();
            node_mut.parent = Some(Rc::downgrade(&new_parent));
        }
        {
            let mut new_parent_mut = new_parent.borrow_mut();
            new_parent_mut.children.push(node_ref.clone());
        }

        // 3. Recompute completion upwards from new parent
        Self::propagate_done_upward(&new_parent);

        true
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

    /// Remove this node and all descendants from the `index` map
    fn remove_from_index_rec(&mut self, node_ref: &NodeRef) {
        let node = node_ref.borrow();
        self.index.remove(&node.id);
        for child in &node.children {
            // Recurse down
            self.remove_from_index_rec(child);
        }
    }

    /// Return `true` if `target` is in the subtree of `root`
    fn is_descendant(root: &NodeRef, target: &NodeRef) -> bool {
        if Rc::ptr_eq(root, target) {
            return true;
        }

        let node = root.borrow();
        for child in &node.children {
            if Self::is_descendant(child, target) {
                // Recurse down
                return true;
            }
        }

        false
    }

    /// Recalculate this node's completion based on its children,
    /// then propagage upwards via parent links.
    fn propagate_done_upward(node_ref: &NodeRef) {
        // Recompuate done for this node, based on its children
        {
            let mut node = node_ref.borrow_mut();
            if !node.children.is_empty() {
                let all_children_done = node
                    .children
                    .iter()
                    .all(|child_ref| child_ref.borrow().done);
                node.done = all_children_done;
            }
        }

        // Now move to the parent
        let parent_weak_opt = {
            let node = node_ref.borrow();
            node.parent.clone()
        };

        if let Some(parent_weak) = parent_weak_opt {
            if let Some(parent_rc) = parent_weak.upgrade() {
                // Proceed to parent only if it still exists
                Self::propagate_done_upward(&parent_rc);
            }
        }
    }
}

/// For `println!("{}", tree);`
impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_pretty(f)
    }
}
