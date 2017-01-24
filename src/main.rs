//! FBX tree viewer.
#![warn(missing_docs)]

extern crate fbxcel;
extern crate gtk;

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use fbx::Attribute;
use fbxcel::parser::binary as fbxbin;
use gtk::prelude::*;
use gtk::{Paned, Orientation, Window, WindowType};
use gtk::{TreeStore, TreeView, ListStore};
use gtk::{Menu, MenuItem, MenuBar};
use gtk::{FileChooserDialog, FileChooserAction, FileFilter};
use gtk::ScrolledWindow;

pub mod fbx;

const WINDOW_TITLE_BASE: &'static str = "FBX tree viewer";


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

    //
    // Menu bar.
    //

    let menu_bar = MenuBar::new();
    let menu_file = MenuItem::new_with_label("File");
    let submenu_file = Menu::new();
    let menu_file_open = MenuItem::new_with_label("Open FBX binary");
    submenu_file.append(&menu_file_open);
    submenu_file.append(&gtk::SeparatorMenuItem::new());
    let menu_file_quit = MenuItem::new_with_label("Quit");
    submenu_file.append(&menu_file_quit);
    menu_file.set_submenu(Some(&submenu_file));
    menu_bar.append(&menu_file);
    root_widget.pack_start(&menu_bar, false, false, 0);

    //
    // FBX tree.
    //

    let node_tree = FbxNodeTree::new();
    let scrolled_node_tree = ScrolledWindow::new(None, None);
    scrolled_node_tree.add(&node_tree.widget);

    //
    // Node data.
    //

    let node_attrs = FbxAttributeTable::new();
    let scrolled_node_attrs = ScrolledWindow::new(None, None);
    scrolled_node_attrs.add(&node_attrs.widget);

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
    let scrolled_logs = ScrolledWindow::new(None, None);
    scrolled_logs.add(&logs.widget);

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


fn load_fbx_binary<P: AsRef<Path>>(path: P, window: &Window, logs: &Logs, node_tree: &FbxNodeTree, node_attrs: &FbxAttributeTable) {
    use std::fs::File;
    use std::io::BufReader;
    use fbxcel::parser::binary::Parser;

    let path = path.as_ref();
    println!("FBX binary path = {}", path.display());
    window.set_title(&format!("{} - {}", WINDOW_TITLE_BASE, path.display()));

    logs.clear();
    node_tree.clear();
    node_attrs.clear();

    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            println!("Cannot open file {}: {}", path.display(), err);
            logs.set_store(&vec![], Some(&err));
            return;
        },
    };
    let mut parser = fbxbin::RootParser::from_seekable(BufReader::new(file));
    let mut open_nodes_iter = Vec::new();
    let mut attr_index = 0;
    let error;
    'load_nodes: loop {
        use fbxcel::parser::binary::Event;

        match parser.next_event() {
            Ok(Event::StartFbx(header)) => {
                let _ = header;
                node_tree.append(None, "(FBX header)", None, 0);
            },
            Ok(Event::EndFbx(result)) => {
                node_tree.append(None, "(FBX footer)", None, 0);
                error = result.err();
                break;
            },
            Ok(Event::StartNode(mut header)) => {
                let tree_iter = node_tree.append(open_nodes_iter.last(), header.name, header.attributes.num_attributes(), attr_index);
                attr_index += header.attributes.num_attributes();
                open_nodes_iter.push(tree_iter);
                'load_attrs: loop {
                    let attr = match header.attributes.next_attribute() {
                        Ok(Some(val)) => val,
                        Ok(None) => break 'load_attrs,
                        Err(err) => {
                            error = Some(err);
                            break 'load_nodes;
                        },
                    };
                    match Attribute::read(attr) {
                        Ok(val) => node_attrs.push_attrs(val),
                        Err(err) => {
                            error = Some(err);
                            break 'load_nodes;
                        },
                    }
                }
            },
            Ok(Event::EndNode) => {
                open_nodes_iter.pop();
            },
            Err(err) => {
                error = Some(err);
                break;
            },
        }
    }
    logs.set_store(parser.warnings(), error.as_ref().map(|e| e as &::std::error::Error));
}


fn create_fbx_binary_chooser<'a, W: Into<Option<&'a Window>>>(window: W) -> FileChooserDialog {
    let file_chooser = FileChooserDialog::new(Some("Open FBX binary file"), window.into(), FileChooserAction::Open);
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
        ("Cancel", gtk::ResponseType::Cancel.into()),
        ("Open", gtk::ResponseType::Ok.into()),
    ]);
    file_chooser
}


#[derive(Debug, Clone)]
struct Logs {
    pub store: TreeStore,
    pub widget: TreeView,
    pub num_entries: Rc<Cell<u64>>,
}

impl Logs {
    /// Creates a new log store and widget.
    pub fn new() -> Self {
        use gtk::{TreeViewColumn, CellRendererText};

        let column_types = &[gtk::Type::U64, gtk::Type::String, gtk::Type::String];
        let store = TreeStore::new(column_types);
        let widget = TreeView::new_with_model(&store);
        widget.set_headers_visible(true);
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            // Right align.
            column.set_alignment(1.0);
            column.set_title("#");
            column.add_attribute(&cell, "text", 0);
            column.set_clickable(true);
            column.set_sort_column_id(0);
            widget.append_column(&column);
        }
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title("severity");
            column.add_attribute(&cell, "text", 1);
            column.set_clickable(true);
            column.set_resizable(true);
            column.set_sort_column_id(1);
            widget.append_column(&column);
        }
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title("description");
            column.add_attribute(&cell, "text", 2);
            column.set_clickable(true);
            column.set_resizable(true);
            column.set_sort_column_id(2);
            widget.append_column(&column);
        }

        Logs {
            store: store,
            widget: widget,
            num_entries: Rc::new(Cell::new(0)),
        }
    }

    pub fn set_store<'a, W: IntoIterator<Item=&'a fbxbin::Warning>>(&self, warnings: W, error: Option<&::std::error::Error>) {
        self.clear();
        for warning in warnings {
            self.append(warning, "warning");
        }

        if let Some(err) = error {
            self.append(err, "Error");
        }
    }

    fn append(&self, err: &::std::error::Error, severity: &str) {
        let mut target = err;
        let mut parent = None;
        let mut i: u64 = self.num_entries.get();
        loop {

            parent = Some(self.store.insert_with_values(parent.as_ref(), None, &[0, 1, 2], &[&i, &severity, &target.to_string()]));
            i += 1;
            match target.cause() {
                Some(err) => target = err,
                None => break,
            }
        }
        self.num_entries.set(i);
    }

    /// Clears internal store.
    pub fn clear(&self) {
        self.store.clear();
        self.num_entries.set(0);
    }
}


#[derive(Debug, Clone)]
struct FbxNodeTree {
    pub store: TreeStore,
    pub widget: TreeView,
}

impl FbxNodeTree {
    /// Creates a new fbx ndoes store and widget.
    pub fn new() -> Self {
        use gtk::{TreeViewColumn, CellRendererText};

        // node name, # of attributes, index of attribute.
        let column_types = &[gtk::Type::String, gtk::Type::U64, gtk::Type::U64];
        let store = TreeStore::new(column_types);
        let widget = TreeView::new_with_model(&store);
        widget.set_grid_lines(gtk::TreeViewGridLines::Vertical);
        widget.set_enable_tree_lines(true);
        widget.set_headers_visible(true);
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title("node name");
            column.add_attribute(&cell, "text", 0);
            column.set_resizable(true);
            widget.append_column(&column);
        }
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title("# of attrs");
            column.add_attribute(&cell, "text", 1);
            column.set_resizable(true);
            widget.append_column(&column);
        }

        FbxNodeTree {
            store: store,
            widget: widget,
        }
    }

    /// Connect events.
    pub fn initialize(&self, node_attrs: &FbxAttributeTable) {
        let node_attrs = node_attrs.clone();
        self.widget.get_selection().connect_changed(move |selection| {
            let (paths, model) = selection.get_selected_rows();
            let descendant_path = match paths.last() {
                Some(path) => path,
                None => {
                    println!("selection has changed but paths is empty");
                    return;
                },
            };
            let tree_iter = match model.get_iter(descendant_path) {
                Some(iter) => iter,
                None => {
                    println!("selection has changed but tree_iter is invalid for path {:?}", descendant_path);
                    return;
                },
            };
            let num_attrs = model.get_value(&tree_iter, 1).get::<u64>().expect("column[1] of `FbxAttributeTable` is not u64");
            let attrs_index = model.get_value(&tree_iter, 2).get::<u64>().expect("column[2] of `FbxAttributeTable` is not u64");
            node_attrs.show_attrs(attrs_index, num_attrs);
        });
    }

    /// Clears internal store.
    pub fn clear(&self) {
        self.store.clear();
    }

    fn append<N: Into<Option<u64>>>(&self, parent: Option<&gtk::TreeIter>, name: &str, num_attrs: N, attr_index: u64) -> gtk::TreeIter {
        self.store.insert_with_values(parent, None, &[0, 1, 2], &[&name, num_attrs.into().as_ref().unwrap_or(&0), &attr_index])
    }
}


#[derive(Debug, Clone)]
struct FbxAttributeTable {
    pub store: ListStore,
    pub widget: TreeView,
    attrs: Rc<RefCell<Vec<Attribute>>>,
}

impl FbxAttributeTable {
    /// Creates a new fbx ndoes store and widget.
    pub fn new() -> Self {
        use gtk::{TreeViewColumn, CellRendererText};

        // index, type, value.
        let column_types = &[gtk::Type::U64, gtk::Type::String, gtk::Type::String];
        let store = ListStore::new(column_types);
        let widget = TreeView::new_with_model(&store);
        widget.set_grid_lines(gtk::TreeViewGridLines::Vertical);
        widget.set_enable_tree_lines(true);
        widget.set_headers_visible(true);
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title("#");
            column.add_attribute(&cell, "text", 0);
            column.set_clickable(true);
            column.set_sort_column_id(0);
            widget.append_column(&column);
        }
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title("type");
            column.add_attribute(&cell, "text", 1);
            column.set_clickable(true);
            column.set_resizable(true);
            column.set_sort_column_id(1);
            widget.append_column(&column);
        }
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title("value");
            column.add_attribute(&cell, "text", 2);
            column.set_resizable(true);
            widget.append_column(&column);
        }
        //widget.get_selection().connect_changed(|selection| {
            //let (paths, model) = selection.get_selected_rows();
            //println!("row selected: {:?}", paths);
        //});

        FbxAttributeTable {
            store: store,
            widget: widget,
            attrs: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Clears internal store.
    pub fn clear(&self) {
        self.store.clear();
        self.attrs.borrow_mut().clear();
    }

    fn push_attrs(&self, attr: Attribute) {
        self.attrs.borrow_mut().push(attr);
    }

    fn show_attrs(&self, attrs_index: u64, num_attrs: u64) {
        self.store.clear();
        for (local_index, attr) in self.attrs.borrow()[attrs_index as usize..(attrs_index + num_attrs) as usize].iter().enumerate() {
            self.append_store(local_index as u64, attr.type_string(), &attr.value_string());
        }
    }

    fn append_store(&self, index: u64, typename: &str, value: &str) -> gtk::TreeIter {
        self.store.insert_with_values(None, &[0, 1, 2], &[&index, &typename, &value])
    }
}
