use core::ops::Deref;

use thunderdome::{Arena, Index};

/// A tree with manages a retained state of `T` for each element.
pub struct RetainedTree<T> {
    inner: Arena<NodeInner<T>>,
    root: Index,
}

// Build tree
// Visit
// Specify action to rebuild part of tree

/// Describes what the visitor should do after visiting this node:
pub enum VisitAction {
    /// Continue on to the next node.
    Continue,

    /// Prune this node.
    Prune,
}

impl<T> RetainedTree<T> {
    pub fn new<Context, Init, Visit>(context: &Context, init: Init, visit: Visit) -> Self
    where
        Init: FnOnce(&Context) -> T,
        Visit: Fn(&Context, Node<'_, T>) -> VisitAction,
    {
        let mut inner = Arena::new();
        let root = init(context);

        let node = NodeInner {
            data: root,
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
        };

        let root = inner.insert(node);

        let mut tree = Self { inner, root };

        tree.visit(context, visit);
        tree
    }

    pub fn visit<Context, Visit>(&mut self, context: &Context, visit: Visit)
    where
        Visit: Fn(&Context, Node<'_, T>) -> VisitAction,
    {
    }

    // TODO: Pruning during iteration and generation of tree components.
    //
    // The idea would be that the parent can generate it's children:
    //
    // Graph node, used to insert elements into the tree.
    //                ||
    //                ||
    //                \/
    //    Fn(&T, &C, Node)   ->   /* Children */
    //        ^   ^                      ^
    //        |    \                     |
    //        |     \                    |
    //      parent   context          children
    //
    // `Children` refers to the generated child nodes.
    //
    // 1. A parent has state to generate children.
    // 2. By virtue of composition, the children also have state to generate their own children.
    //
    // Issues:
    // - Returning a RetainedTree is not ideal for each parent and it's children recursively.
    //   - SOLUTION: When pruned, the node passed when creating children will add the branches to the tree.
    // - A menu would require rebuilding every child
    //   - SOLUTION: Offscreen branches. Offscreen branches are a set of nodes and children in the tree.
    //     However these are not considered to be part of the "visible" hierarchy.
    // - Async?
    //   - UNDECIDED: Invalidate a node on completion of a Future?
}

pub struct Node<'a, T> {
    inner: &'a mut Arena<NodeInner<T>>,
    index: Index,
}

impl<'a, T> Node<'a, T> {
    #[must_use]
    pub fn push_child<'b>(&'b mut self, data: T) -> Node<'b, T>
    where
        'a: 'b,
    {
        let index = insert_new_child(self.inner, data, self.index);

        Node {
            inner: self.inner,
            index,
        }
    }
}

impl<T> Deref for Node<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner.get(self.index).unwrap().data
    }
}

pub struct DepthFirst<'a, T> {
    tree: &'a RetainedTree<T>,
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

struct NodeInner<T> {
    data: T,
    parent: Option<Index>,
    prev_sibling: Option<Index>,
    next_sibling: Option<Index>,
    first_child: Option<Index>,
    last_child: Option<Index>,
}

impl<T> RetainedTree<T> {
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

fn get_next_node<T>(arena: &Arena<NodeInner<T>>, index: Index) -> Option<Index> {
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

fn insert_new_child<T>(arena: &mut Arena<NodeInner<T>>, data: T, parent: Index) -> Index {
    // Get the previous sibling of the new node if possible
    let parent_node = arena.get_mut(parent).unwrap();
    let prev_sibling = parent_node.last_child;

    let index = arena.insert(NodeInner {
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
