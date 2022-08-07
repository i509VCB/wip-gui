use thunderdome::{Arena, Index};

pub struct Tree<T> {
    inner: Arena<Node<T>>,
    root: Option<Index>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self {
            inner: Arena::new(),
            root: None,
        }
    }

    pub fn insert(&mut self, data: T) -> Slot<'_, T> {
        assert!(self.root.is_none(), "Root node already created");

        let index = self.inner.insert(Node {
            data,
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
        });

        self.root = Some(index);

        Slot {
            inner: &mut self.inner,
            index,
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            tree: self,
            next: self.root,
            first: true,
        }
    }

    // TODO: Visitor which can mutate or invalidate branches of the tree
}

pub struct Iter<'a, T> {
    tree: &'a Tree<T>,
    next: Option<Index>,
    first: bool,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        let node = self.tree.inner.get(current)?;

        // Get the next sibling
        if let Some(next_sibling) = node.next_sibling {
            self.next = Some(next_sibling);
        } else if let Some(parent) = node.parent {
            // Otherwise find the next sibling on the parent.
            if let Some(parent) = self.tree.inner.get(parent) {
                // Traverse until we find the current node, and then take the next node.
                if let Some(mut next) = parent.first_child {
                    loop {
                        if next == current {
                            self.next = Some(next);
                            break;
                        }

                        // If the node does not match, test the next sibling
                        if let Some(next_node) = self.tree.inner.get(next) {
                            if let Some(next_sibling) = next_node.next_sibling {
                                next = next_sibling;
                                continue;
                            }
                        }

                        // The end of the iterator.
                        break;
                    }
                }
            }
        } else if self.first {
            // FIXME: Iterator never finishes
            self.next = node.first_child;
        }

        self.first = false;

        Some(&node.data)
    }
}

pub struct Slot<'a, T> {
    inner: &'a mut Arena<Node<T>>,
    index: Index,
}

impl<'a, T> Slot<'a, T> {
    #[must_use]
    pub fn insert_child<'b>(&'b mut self, data: T) -> Slot<'b, T>
    where
        'a: 'b,
    {
        let index = insert_new_child(&mut self.inner, data, self.index);

        Slot {
            inner: self.inner,
            index,
        }
    }
}

struct Node<T> {
    data: T,
    parent: Option<Index>,
    prev_sibling: Option<Index>,
    next_sibling: Option<Index>,
    first_child: Option<Index>,
    last_child: Option<Index>,
}

fn insert_new_child<T>(arena: &mut Arena<Node<T>>, data: T, parent: Index) -> Index {
    // Get the previous sibling of the new node if possible
    let parent_node = arena.get_mut(parent).unwrap();
    let prev_sibling = parent_node.last_child;

    let index = arena.insert(Node {
        data,
        parent: Some(parent),
        prev_sibling,
        next_sibling: None,
        first_child: None,
        last_child: None,
    });

    // The parent node has gone out of scope by inserting the new node.
    let parent_node = arena.get_mut(parent).unwrap();

    // If this is the parent node's first child node, update the first child as well.
    if parent_node.first_child.is_none() {
        parent_node.first_child = Some(index);
    }

    // Add the new last child
    parent_node.last_child = Some(index);

    // Update the next sibling of the previous sibling
    if let Some(prev_sibling) = prev_sibling {
        let prev_sibling = arena.get_mut(prev_sibling).unwrap();
        prev_sibling.next_sibling = Some(index);
    }

    index
}

#[cfg(test)]
mod tests {
    use super::Tree;

    #[test]
    fn test_insert() {
        let mut tree = Tree::<u32>::new();
        let _ = tree.insert(0);
        assert!(tree.root.is_some());
    }

    #[test]
    fn two_children() {
        let mut tree = Tree::<u32>::new();

        {
            let mut node = tree.insert(0);
            let _ = node.insert_child(2);
            let _ = node.insert_child(3);
            let _ = node.insert_child(4);
        }

        assert!(tree.root.is_some());
        assert_eq!(tree.inner.len(), 4);

        // Get root node
        let root = tree.inner.get(tree.root.unwrap()).unwrap();

        // Assert the first child and last child are not None
        assert!(root.first_child.is_some());
        assert!(root.last_child.is_some());

        // Assert the first and last child are not equal since more than one child node exists.
        assert_ne!(root.first_child, root.last_child);

        // Traverse to the 1st child and ensure relations are correct.
        let first_child_node = tree.inner.get(root.first_child.unwrap()).unwrap();
        assert_eq!(first_child_node.data, 2);

        // Parent
        assert_eq!(first_child_node.parent, tree.root);
        // First child of a parent will have no previous sibling
        assert!(first_child_node.prev_sibling.is_none());
        // But can have a next sibling
        assert!(first_child_node.next_sibling.is_some());

        // 2nd child
        let second_child_node = tree
            .inner
            .get(first_child_node.next_sibling.unwrap())
            .unwrap();
        assert_eq!(second_child_node.data, 3);

        // Parent
        assert_eq!(second_child_node.parent, tree.root);
        // 2nd child's has a previous and next sibling
        assert!(second_child_node.prev_sibling.is_some());
        assert!(second_child_node.next_sibling.is_some());
        // The previous sibling must be the first child
        assert_eq!(second_child_node.prev_sibling, root.first_child);

        // The last child of the root should be the next sibling of the second child
        assert_eq!(second_child_node.next_sibling, root.last_child);

        // Last child
        let last_child_node = tree
            .inner
            .get(second_child_node.next_sibling.unwrap())
            .unwrap();
        assert_eq!(last_child_node.data, 4);

        assert_eq!(last_child_node.parent, tree.root);
        // The last child should have no next sibling
        assert!(last_child_node.next_sibling.is_none());
        assert_eq!(last_child_node.prev_sibling, first_child_node.next_sibling);

        let mut iter = tree.iter();
        assert_eq!(Some(&0), iter.next());
        assert_eq!(Some(&2), iter.next());
        assert_eq!(Some(&3), iter.next());
        assert_eq!(Some(&4), iter.next());
        // FIXME: Iterator never ends
        assert_eq!(None, iter.next());
    }
}
