//! FBX attributes table.

use std::{cell::RefCell, rc::Rc};

use crate::fbx::Attribute;

use glib::Type;
use gtk::{prelude::*, ListStore, TreeView};

/// FBX attributes table.
#[derive(Debug, Clone)]
pub struct FbxAttributeTable {
    store: ListStore,
    widget: TreeView,
    attrs: Rc<RefCell<Vec<Attribute>>>,
}

impl FbxAttributeTable {
    /// Creates a new fbx ndoes store and widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears internal store.
    pub fn clear(&self) {
        self.store.clear();
        self.attrs.borrow_mut().clear();
    }

    /// Pushes the given attribute.
    pub fn push_attribute(&self, attr: Attribute) {
        self.attrs.borrow_mut().push(attr);
    }

    /// Show the attributes.
    pub fn show_attrs(&self, attrs_index: u64, num_attrs: u64) {
        self.store.clear();
        for (local_index, attr) in self.attrs.borrow()
            [attrs_index as usize..(attrs_index + num_attrs) as usize]
            .iter()
            .enumerate()
        {
            self.append_store(local_index as u64, attr.type_string(), &attr.value_string());
        }
    }

    fn append_store(&self, index: u64, typename: &str, value: &str) -> gtk::TreeIter {
        self.store
            .insert_with_values(None, &[0, 1, 2], &[&index, &typename, &value])
    }

    /// Returns a reference to the `TreeView`.
    pub fn widget(&self) -> &TreeView {
        &self.widget
    }
}

impl Default for FbxAttributeTable {
    fn default() -> Self {
        use gtk::{CellRendererText, TreeViewColumn};

        // index, type, value.
        let column_types = &[Type::U64, Type::String, Type::String];
        let store = ListStore::new(column_types);
        let widget = TreeView::with_model(&store);
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

        Self {
            store,
            widget,
            attrs: Rc::new(RefCell::new(Vec::new())),
        }
    }
}
