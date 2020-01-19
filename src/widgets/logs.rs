//! Logs.

use std::{cell::Cell, rc::Rc};

use fbxcel::pull_parser as fbxbin;
use glib::Type;
use gtk::{prelude::*, TreeStore, TreeView};

/// Logs widget.
#[derive(Debug, Clone)]
pub struct Logs {
    store: TreeStore,
    widget: TreeView,
    num_entries: Rc<Cell<u64>>,
}

impl Logs {
    /// Creates a new log store and widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the given warnings and errors to be shown.
    pub fn set_store<
        'a,
        W: IntoIterator<Item = &'a (fbxbin::Warning, fbxbin::SyntacticPosition)>,
    >(
        &self,
        warnings: W,
        error: Option<&(dyn std::error::Error + 'static)>,
    ) {
        self.clear();
        for (warning, syn_pos) in warnings {
            self.append(warning, Some(syn_pos), "warning");
        }

        if let Some(err) = error {
            let syn_pos = err
                .downcast_ref::<fbxbin::Error>()
                .and_then(|parser_error| parser_error.position());
            self.append(err, syn_pos, "Error");
        }
    }

    fn append(
        &self,
        err: &dyn std::error::Error,
        syn_pos: Option<&fbxbin::SyntacticPosition>,
        severity: &str,
    ) {
        let mut target = err;
        let mut parent = None;
        let mut i: u64 = self.num_entries.get();
        loop {
            let syn_pos = syn_pos.map_or_else(String::new, |pos| format!("{:?}", pos));
            parent = Some(self.store.insert_with_values(
                parent.as_ref(),
                None,
                &[0, 1, 2, 3],
                &[&i, &severity, &target.to_string(), &syn_pos],
            ));
            i += 1;
            match target.source() {
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

    /// Returns a reference to the `TreeView`.
    pub fn widget(&self) -> &TreeView {
        &self.widget
    }
}

impl Default for Logs {
    fn default() -> Self {
        use gtk::{CellRendererText, TreeViewColumn};

        // Error and warning index, severity, description, syntactic position
        let column_types = &[Type::U64, Type::String, Type::String, Type::String];
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
        {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title("position");
            column.add_attribute(&cell, "text", 3);
            column.set_clickable(true);
            column.set_resizable(true);
            column.set_sort_column_id(3);
            widget.append_column(&column);
        }

        Self {
            store,
            widget,
            num_entries: Rc::new(Cell::new(0)),
        }
    }
}
