// src/main.rs

mod tree;

use crate::tree::Tree;
use std::io::{self, Write};

fn main() {
    let mut tree = Tree::new();

    println!("treedesk-rs REPL");
    println!("Type 'help' for commands, 'quit' to exit.\n");

    loop {
        print!("> ");

        // flush so the prompt actually shows before we read
        io::stdout().flush().unwrap();

        let mut line = String::new();
        let read = io::stdin().read_line(&mut line);
        match read {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Handle quit commands early
        if line.eq_ignore_ascii_case("quit") || line.eq_ignore_ascii_case("exit") {
            break;
        }

        // Dispatch command
        match handle_command(line, &mut tree) {
            Ok(()) => {}
            Err(msg) => eprintln!("Error: {}", msg),
        }
    }

    println!("Goodbye! >_<");
}

fn handle_command(line: &str, tree: &mut Tree) -> Result<(), String> {
    // Split once into command + rest
    let mut parts = line.splitn(2, char::is_whitespace);
    let cmd = parts.next().unwrap();
    let args = parts.next().unwrap_or("").trim();

    match cmd {
        "help" => {
            print_help();
        }

        "root" => {
            if args.is_empty() {
                return Err("usage: root <title>".into());
            }
            let id = tree.add_root(args.to_string());
            println!("Added root node with id {}", id);
        }

        "show" => {
            println!("{}", tree);
        }

        "child" => {
            // expect: child <parent_id> <title>
            let mut parts = args.splitn(2, char::is_whitespace);
            let parent_id_str = parts.next().ok_or("usage: child <parent_id> <title>")?;
            let title = parts
                .next()
                .ok_or("usage: child <parent_id> <title>")?
                .trim();
            if title.is_empty() {
                return Err("title cannot be empty".into());
            }

            let parent_id: u32 = parent_id_str
                .parse()
                .map_err(|_| "parent_id must be a number".to_string())?;

            match tree.add_child(parent_id, title.to_string()) {
                Some(id) => {
                    println!("Added child node with id {}", id);
                }
                None => {
                    return Err(format!("parent_id {} not found", parent_id));
                }
            }
        }

        "toggle" => {
            let id_str = args;
            if id_str.is_empty() {
                return Err("usage: toggle <id>".into());
            }

            let id: u32 = id_str
                .parse()
                .map_err(|_| "id must be a number".to_string())?;

            match tree.toggle(id) {
                true => {
                    println!("Toggled done flag for node {}", id);
                }
                false => {
                    return Err(format!("id {} not found", id));
                }
            }
        }

        "delete" => {
            let id_str = args;
            if id_str.is_empty() {
                return Err("usage: delete <id>".into());
            }

            let id: u32 = id_str
                .parse()
                .map_err(|_| "id must be a number".to_string())?;

            match tree.delete(id) {
                true => {
                    println!("Deleted node {}", id);
                }
                false => {
                    return Err(format!("id {} not found", id));
                }
            }
        }

        "move" => {
            // expect: move <id> <new_parent_id>
            let mut parts = args.split_whitespace();
            let id_str = parts.next().ok_or("usage: move <id> <new_parent_id>")?;
            let parent_id_str = parts.next().ok_or("usage: move <id> <new_parent_id>")?;

            let id: u32 = id_str
                .parse()
                .map_err(|_| "id must be a number".to_string())?;

            let new_parent_id: u32 = parent_id_str
                .parse()
                .map_err(|_| "new_parent_id must be a number".to_string())?;

            if tree.move_node(id, new_parent_id) {
                println!("Moved node {} under new parent {}", id, new_parent_id);
            } else {
                return Err(format!(
                    "failed to move node {} under new parent {} (check ids and for cycles)",
                    id, new_parent_id
                ));
            }
        }

        "get" => {
            // simple peek: get <id>
            let id_str = args;
            if id_str.is_empty() {
                return Err("usage: get <id>".into());
            }

            let id: u32 = id_str
                .parse()
                .map_err(|_| "id must be a number".to_string())?;
            if let Some(node_ref) = tree.get(id) {
                let node = node_ref.borrow();
                println!(
                    "[{}] {} (id: {})",
                    if node.done { "x" } else { " " },
                    node.title,
                    node.id
                );
                println!("children: {}", node.children.len());
            } else {
                return Err(format!("id {} not found", id));
            }
        }

        other => {
            return Err(format!("unknown command: {other} (try 'help')"));
        }
    }

    Ok(())
}

fn print_help() {
    println!(
        "\
Commands:
  show
      Show the whole tree.

  root <title>
      Add a new root node.

  child <parent_id> <title>
      Add a child under the given parent.

  toggle <id>
      Toggle the 'done' flag for a node. Auto-completes parents if all children done.

  delete <id>
      Delete a node and its subtree.

  move <id> <new_parent_id>
      Move a node to a new parent (fails if it would create a cycle).

  get <id>
      Show a single node and how many children it has.

  help
      Show this help.

  quit | exit
      Leave the REPL.
"
    );
}
