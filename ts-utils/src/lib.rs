use tree_sitter::{Node, TreeCursor};

pub struct ChildCursor<'a> {
    cursor: TreeCursor<'a>,
    done: bool,
    first: bool,
}

impl<'a> ChildCursor<'a> {
    pub fn new(node: &Node<'a>) -> Self {
        let mut cursor = node.walk();
        let done = !cursor.goto_first_child();

        Self {
            cursor,
            done,
            first: true,
        }
    }
}

impl<'a> Iterator for ChildCursor<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if self.first {
            self.first = false;
            return Some(self.cursor.node());
        }

        if self.cursor.goto_next_sibling() {
            return Some(self.cursor.node());
        }

        self.done = true;
        None
    }
}

pub trait NodeChildCursorExt<'a> {
    fn children_cursor(&self) -> ChildCursor<'a>;
}

impl<'a> NodeChildCursorExt<'a> for Node<'a> {
    fn children_cursor(&self) -> ChildCursor<'a> {
        ChildCursor::new(self)
    }
}
