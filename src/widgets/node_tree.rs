//! FBX node tree widget.

use glib::Type;
use gtk::{prelude::*, TreeStore, TreeView};

use crate::FbxAttributeTable;

/// FBX node tree widget.
#[derive(Debug, Clone)]
pub struct FbxNodeTree {
    store: TreeStore,
    widget: TreeView,
}

impl FbxNodeTree {
    /// Creates a new fbx ndoes store and widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Connect events.
    pub fn initialize(&self, node_attrs: &FbxAttributeTable) {
        let node_attrs = node_attrs.clone();
        self.widget
            .get_selection()
            .connect_changed(move |selection| {
                let (paths, model) = selection.get_selected_rows();
                let descendant_path = match paths.last() {
                    Some(path) => path,
                    None => {
                        println!("selection has changed but paths is empty");
                        return;
                    }
                };
                let tree_iter = match model.get_iter(descendant_path) {
                    Some(iter) => iter,
                    None => {
                        println!(
                            "selection has changed but tree_iter is invalid for path {:?}",
                            descendant_path
                        );
                        return;
                    }
                };
                let num_attrs = model
                    .get_value(&tree_iter, 1)
                    .get_some::<u64>()
                    .expect("column[1] of `FbxAttributeTable` is not u64");
                let attrs_index = model
                    .get_value(&tree_iter, 2)
                    .get_some::<u64>()
                    .expect("column[2] of `FbxAttributeTable` is not u64");
                node_attrs.show_attrs(attrs_index, num_attrs);
            });
    }

    /// Clears internal store.
    pub fn clear(&self) {
        self.store.clear();
    }

    /// Appends the given node.
    pub fn append<N: Into<Option<u64>>>(
        &self,
        parent: Option<&gtk::TreeIter>,
        name: &str,
        num_attrs: N,
        attr_index: u64,
    ) -> gtk::TreeIter {
        self.store.insert_with_values(
            parent,
            None,
            &[0, 1, 2],
            &[&name, num_attrs.into().as_ref().unwrap_or(&0), &attr_index],
        )
    }

    /// Returns a reference to the `TreeView`.
    pub fn widget(&self) -> &TreeView {
        &self.widget
    }
}

impl Default for FbxNodeTree {
    fn default() -> Self {
        use gtk::{CellRendererText, TreeViewColumn};

        // node name, # of attributes, index of attribute.
        let column_types = &[Type::String, Type::U64, Type::U64];
        let store = TreeStore::new(column_types);
        let widget = TreeView::with_model(&store);
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

        Self { store, widget }
    }
}
