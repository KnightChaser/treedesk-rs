// src/main.rs

mod tree;

use crate::tree::Tree;

fn main() {
    let mut t = Tree::new();

    let inbox = t.add_root("Inbox");
    let work = t.add_root("Work");
    let personal = t.add_root("Personal");

    let r1 = t.add_child(inbox, "Buy milk");
    let r2 = t.add_child(inbox, "Finish Rust book");
    let r3 = t.add_child(work, "Write report");
    let r4 = t.add_child(personal, "Call mom");

    // mark some as done
    if let Some(id) = r1 {
        t.toggle(id);
    }
    if let Some(id) = r3 {
        t.toggle(id);
    }

    if let Some(id) = r2 {
        // nested children just to test depth
        let _ = t.add_child(id, "Take notes on ownership");
    }

    println!("{}", t);
    println!("========================");

    if let Some(id) = r4 {
        t.toggle(id);
    }

    println!("{}", t);
}
