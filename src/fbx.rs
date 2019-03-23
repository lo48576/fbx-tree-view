//! FBX data.

use std::{cell::RefCell, path::Path, rc::Rc};

use fbxcel::pull_parser::{self as fbxbin, any::AnyParser};
use gtk::{prelude::*, Window};

use crate::{
    widgets::{FbxAttributeTable, FbxNodeTree, Logs},
    WINDOW_TITLE_BASE,
};

pub use self::attribute::{Attribute, AttributeLoader};

mod attribute;

/// Loads the given FBX binary file.
pub fn load_fbx_binary<P: AsRef<Path>>(
    path: P,
    window: &Window,
    logs: &Logs,
    node_tree: &FbxNodeTree,
    node_attrs: &FbxAttributeTable,
) {
    use std::fs::File;
    use std::io::BufReader;

    let path = path.as_ref();
    println!("FBX binary path = {}", path.display());
    window.set_title(&format!("{} - {}", WINDOW_TITLE_BASE, path.display()));

    logs.clear();
    node_tree.clear();
    node_attrs.clear();

    let mut file = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            println!("Cannot open file {}: {}", path.display(), err);
            logs.set_store(&vec![], Some(&err));
            return;
        }
    };
    let parser = match fbxbin::any::from_seekable_reader(&mut file) {
        Ok(v) => v,
        Err(err) => {
            println!("Cannot open file {} as FBX binary: {}", path.display(), err);
            logs.set_store(&vec![], Some(&err));
            return;
        }
    };
    node_tree.append(None, "(FBX header)", None, 0);

    println!(
        "FBX version: {}.{}",
        parser.fbx_version().major(),
        parser.fbx_version().minor()
    );

    match parser {
        AnyParser::V7400(mut parser) => {
            let warnings = Rc::new(RefCell::new(Vec::new()));
            {
                let warnings = Rc::downgrade(&warnings);
                parser.set_warning_handler(move |warning, syn_pos| {
                    if let Some(rc) = warnings.upgrade() {
                        rc.borrow_mut().push((warning, syn_pos.clone()));
                    }
                    Ok(())
                });
            }
            match load_fbx_binary_v7400(parser, node_tree, node_attrs) {
                Ok(()) => {
                    logs.set_store(warnings.borrow().iter(), None);
                }
                Err(err) => {
                    println!("Failed to parse FBX file: {}", err);
                    logs.set_store(warnings.borrow().iter(), Some(&err));
                    return;
                }
            }
        }
        parser => {
            let ver = format!(
                "{}.{}",
                parser.fbx_version().major(),
                parser.fbx_version().minor()
            );
            println!("Unsupported FBX version: {}", ver);
            let err: Box<dyn std::error::Error> =
                format!("Unsupported FBX version: {}", ver).into();
            logs.set_store(&vec![], Some(err.as_ref()));
            return;
        }
    }
}

fn load_fbx_binary_v7400<R: fbxbin::ParserSource>(
    mut parser: fbxbin::v7400::Parser<R>,
    node_tree: &FbxNodeTree,
    node_attrs: &FbxAttributeTable,
) -> fbxbin::Result<()> {
    let mut open_nodes_iter = Vec::new();
    let mut attr_index = 0;

    'load_nodes: loop {
        use fbxbin::v7400::*;

        match parser.next_event()? {
            Event::StartNode(node) => {
                let name = node.name().to_owned();
                let mut attributes = node.attributes();
                let tree_iter = node_tree.append(
                    open_nodes_iter.last(),
                    &name,
                    attributes.total_count(),
                    attr_index,
                );
                attr_index += attributes.total_count();
                open_nodes_iter.push(tree_iter);
                while let Some(attr) = attributes.load_next(AttributeLoader)? {
                    node_attrs.push_attribute(attr);
                }
            }
            Event::EndNode => {
                open_nodes_iter.pop();
            }
            Event::EndFbx(footer_res) => {
                node_tree.append(None, "(FBX footer)", None, 0);
                let _ = footer_res?;
                break 'load_nodes;
            }
        }
    }

    Ok(())
}
