//! FBX tree viewer.
#![warn(missing_docs)]

extern crate fbxcel;
extern crate gdk;
extern crate gtk;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use fbxcel::pull_parser::{self as fbxbin, any::AnyParser};
use gtk::prelude::*;
use gtk::ScrolledWindow;
use gtk::{AccelFlags, AccelGroup, WidgetExt};
use gtk::{FileChooserAction, FileChooserDialog, FileFilter};
use gtk::{Menu, MenuBar, MenuItem};
use gtk::{Orientation, Paned, Window, WindowType};

use crate::widgets::{FbxAttributeTable, FbxNodeTree, Logs};

pub mod fbx;
pub mod widgets;

const WINDOW_TITLE_BASE: &str = "FBX tree viewer";

fn main() {
    gtk::init().expect("Failed to initialize GTK");

    let window_width = 800;
    let window_height = 600;

    let window = Window::new(WindowType::Toplevel);
    window.set_title(WINDOW_TITLE_BASE);
    window.set_default_size(window_width, window_height);

    //
    // Window root.
    //

    let root_widget = gtk::Box::new(Orientation::Vertical, 0);
    window.add(&root_widget);
    let accel_group = AccelGroup::new();
    window.add_accel_group(&accel_group);

    //
    // Menu bar.
    //

    let menu_bar = MenuBar::new();
    let menu_file = MenuItem::new_with_mnemonic("_File");
    let submenu_file = Menu::new();
    let menu_file_open = MenuItem::new_with_mnemonic("_Open FBX binary");
    submenu_file.append(&menu_file_open);
    submenu_file.append(&gtk::SeparatorMenuItem::new());
    let menu_file_quit = MenuItem::new_with_mnemonic("_Quit");
    submenu_file.append(&menu_file_quit);
    menu_file.set_submenu(Some(&submenu_file));
    menu_bar.append(&menu_file);
    root_widget.pack_start(&menu_bar, false, false, 0);

    {
        use gdk::enums::key;
        menu_file_open.add_accelerator(
            "activate",
            &accel_group,
            key::O,
            gdk::ModifierType::CONTROL_MASK,
            AccelFlags::VISIBLE,
        );
        menu_file_quit.add_accelerator(
            "activate",
            &accel_group,
            key::Q,
            gdk::ModifierType::CONTROL_MASK,
            AccelFlags::VISIBLE,
        );
    }

    //
    // FBX tree.
    //

    let node_tree = FbxNodeTree::new();
    let scrolled_node_tree = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    scrolled_node_tree.add(node_tree.widget());

    //
    // Node data.
    //

    let node_attrs = FbxAttributeTable::new();
    let scrolled_node_attrs = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    scrolled_node_attrs.add(node_attrs.widget());

    node_tree.initialize(&node_attrs);

    //
    // FBX tree and node data.
    //

    let fbx_data_pane = Paned::new(Orientation::Horizontal);
    fbx_data_pane.add1(&scrolled_node_tree);
    fbx_data_pane.add2(&scrolled_node_attrs);
    fbx_data_pane.set_position(window_width / 5 * 4);

    //
    // Warnings and errors.
    //

    let logs = Logs::new();
    let scrolled_logs = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    scrolled_logs.add(logs.widget());

    //
    // Main region of the window.
    //

    let content_pane = Paned::new(Orientation::Vertical);
    content_pane.add1(&fbx_data_pane);
    content_pane.add2(&scrolled_logs);
    content_pane.set_wide_handle(true);
    content_pane.set_position(window_height / 5 * 4);

    root_widget.pack_start(&content_pane, true, true, 0);

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    {
        let window = window.clone();
        let fbx_binary_chooser = create_fbx_binary_chooser(&window.clone());
        let logs = logs.clone();
        let node_tree = node_tree.clone();
        menu_file_open.connect_activate(move |_| {
            if fbx_binary_chooser.run() == gtk::ResponseType::Ok.into() {
                if let Some(filename) = fbx_binary_chooser.get_filename() {
                    load_fbx_binary(filename, &window, &logs, &node_tree, &node_attrs);
                }
            }
            fbx_binary_chooser.hide();
        });
    }
    menu_file_quit.connect_activate(move |_| {
        gtk::main_quit();
    });

    gtk::main();
}

fn load_fbx_binary<P: AsRef<Path>>(
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

fn create_fbx_binary_chooser<'a, W: Into<Option<&'a Window>>>(window: W) -> FileChooserDialog {
    let file_chooser = FileChooserDialog::new(
        Some("Open FBX binary file"),
        window.into(),
        FileChooserAction::Open,
    );
    {
        let fbx_filter = FileFilter::new();
        fbx_filter.set_name(Some("FBX files"));
        fbx_filter.add_pattern("*.fbx");
        fbx_filter.add_pattern("*.FBX");
        file_chooser.add_filter(&fbx_filter);
    }
    {
        let all_filter = FileFilter::new();
        all_filter.set_name(Some("All files"));
        all_filter.add_pattern("*");
        file_chooser.add_filter(&all_filter);
    }
    file_chooser.add_buttons(&[
        ("Cancel", gtk::ResponseType::Cancel),
        ("Open", gtk::ResponseType::Ok),
    ]);
    file_chooser
}

fn load_fbx_binary_v7400<R: fbxbin::ParserSource>(
    mut parser: fbxbin::v7400::Parser<R>,
    node_tree: &FbxNodeTree,
    node_attrs: &FbxAttributeTable,
) -> fbxbin::Result<()> {
    let mut open_nodes_iter = Vec::new();
    let mut attr_index = 0;

    'load_nodes: loop {
        use crate::fbxbin::v7400::*;

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
                while let Some(attr) = attributes.load_next(fbx::AttributeLoader)? {
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
