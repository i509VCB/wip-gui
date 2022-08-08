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

    /// Returns an iterator which visits the tree in depth first order.
    ///
    /// Depth first search will try to explore the furthest point of a branch before backtracking to other
    /// branches.
    pub fn depth_first(&self) -> DepthFirst<'_, T> {
        DepthFirst {
            tree: self,
            next: self.root,
        }
    }

    // TODO: Iterator which goes up the tree from the last node.

    // TODO: Visitor which can mutate or invalidate branches of the tree
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
        let index = insert_new_child(self.inner, data, self.index);

        Slot {
            inner: self.inner,
            index,
        }
    }
}

pub struct DepthFirst<'a, T> {
    tree: &'a Tree<T>,
    next: Option<Index>,
}

impl<'a, T> Iterator for DepthFirst<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        let node = self.tree.inner.get(current)?;

        // Get the next node
        self.next = get_next_node(&self.tree.inner, current);
        Some(&node.data)
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

impl<T> Tree<T> {
    /// Returns the index of the last node in the tree.
    fn get_last(&self) -> Option<Index> {
        self.root.map(|mut next| {
            loop {
                let node = self.inner.get(next).unwrap();

                // Get the next last child
                if let Some(child) = node.last_child {
                    next = child;
                    continue;
                }

                // Reached the last node in the tree.
                break next;
            }
        })
    }
}

fn get_next_node<T>(arena: &Arena<Node<T>>, index: Index) -> Option<Index> {
    // Try the child of the node
    let node = arena.get(index)?;

    node
        // First child
        .first_child
        // Otherwise try the next sibling
        .or(node.next_sibling)
        .or_else(|| {
            // Check if this node has a parent
            if let Some(mut parent) = node.parent {
                loop {
                    let parent_node = arena.get(parent).expect("Parent must exist");

                    // Try the next sibling of the parent
                    //                grandparent
                    //               /           \
                    //         parent ----------> next
                    //        /
                    // current
                    if let Some(next) = parent_node.next_sibling {
                        return Some(next);
                    }

                    // grandparent
                    // ^
                    // |
                    // parent
                    // ^
                    // |
                    // current
                    if let Some(next) = parent_node.parent {
                        parent = next;
                        // The next iteration of the loop will try the sibling and next grandparent.
                        continue;
                    }

                    // We have reached the end of the tree and therefore no more nodes exist.
                    break;
                }
            }

            // We have reached the end of the tree
            None
        })
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
    fn depth_first_branches() {
        let mut tree = Tree::<u32>::new();
        let mut node = tree.insert(0);

        /*
            0
           / \
          1   4
         / \
        2   3
        */

        {
            let mut node = node.insert_child(1);

            let _ = node.insert_child(2);
            let _ = node.insert_child(3);
        }

        let _ = node.insert_child(4);

        assert_eq!(tree.inner.len(), 5);

        let mut iter = tree.depth_first();

        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn depth_first_line() {
        /*
          0
          |
          1
          |
          2
          |
          3
          |
          4
        */
        let mut tree = Tree::<u32>::new();
        let mut node = tree.insert(0);
        let mut node = node.insert_child(1);
        let mut node = node.insert_child(2);
        let mut node = node.insert_child(3);
        let _ = node.insert_child(4);

        let mut iter = tree.depth_first();

        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn depth_first_wide_branch() {
        /*
              0
            __|__
           /  |  \
          1   6   7
          |       |
         / \      8
         2 4
         | |
         3 5
        */
        let mut tree = Tree::<u32>::new();
        let mut node = tree.insert(0);

        {
            let mut node = node.insert_child(1);

            {
                let mut node = node.insert_child(2);
                let _ = node.insert_child(3);
            }

            let mut node = node.insert_child(4);
            let _ = node.insert_child(5);
        }

        let _ = node.insert_child(6);

        {
            let mut node = node.insert_child(7);
            let _ = node.insert_child(8);
        }

        let mut iter = tree.depth_first();

        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), Some(&6));
        assert_eq!(iter.next(), Some(&7));
        assert_eq!(iter.next(), Some(&8));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn depth_first_deep_and_wide() {
        /*
              0
              |
              1
         _____|__
        /  |  |  \
        2  4  7   8
        |  |
        3  5
           |
           6
        */
        let mut tree = Tree::<u32>::new();
        let mut node = tree.insert(0);
        let mut node = node.insert_child(1);

        {
            let mut node = node.insert_child(2);
            let _ = node.insert_child(3);
        }

        {
            let mut node = node.insert_child(4);
            let mut node = node.insert_child(5);
            let _ = node.insert_child(6);
        }

        let _ = node.insert_child(7);
        let _ = node.insert_child(8);

        let mut iter = tree.depth_first();

        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), Some(&6));
        assert_eq!(iter.next(), Some(&7));
        assert_eq!(iter.next(), Some(&8));
        assert_eq!(iter.next(), None);
    }
}
