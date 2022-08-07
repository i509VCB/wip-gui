#![no_std]

extern crate alloc;

mod tree;

pub mod view;

use alloc::vec::Vec;
use view::View;

// Discussion: When do we stop UI?
//
// The problem:
//
// Ui as a type is extremely hard to extend when it is a monolithic blob.
//
// Ideas:
//
// - Split out Layout and widget info
//   - Would split out some of the code related to rendering and layout more backend independent.
//   - Makes event dispatching code less easy to work with.
//   - Some event dispatching requires the layout info to even work properly (click event for example).
//     - SwiftUI does not provide position info or context for input events that are dispatched:
//       - Clicking
//       - Drag and drop
//       - PKCanvasViewDelegate (Pencil, tablet like scenario)
//         - This one is interesting, we don't actually get any events about the content, just what changed.
//         - PKCanvasView does let us create an image.
//       - Gestures
//       - All views (SubmitTriggers = .text)
//         - onSubmit states when entry is completed.
//
//     - gtk4 does provide position info and context for input events that are dispatched:
//       - EventControllerMotion
//       - EventControllerScroll
//       - PadController
//         - Activation with no data, ask the controller for context
//         - Says these events are only delivered to windows?
//       - DragSource (This includes selection of text)
//       - DropTarget
//       - GestureXX types
//       - EventControllerKey (IME and Keyboard events)
//
//     - Events I don't know yet
//       - Hovering over text? Might rely on popups that could be implicit to a widget?
//
//     - Which do we choose?
//       - For 99% of events the precise position info is overkill.
//         - You don't need to know that the exact center or left side of a button was pressed,
//           just that it was pressed.
//       - Surfaces which want tablet input need precise position info
//       - Internally list/stack views need position info to function properly.
//
//       - Precise position info is necessary
//         - Convention: Expose simple event types, pressed, scrolled, clicked
//         - Convention: Where precise event info is needed, provide precise event info (such as tablet input)
//
// - Positioning info IS needed.
//   - More than likely we need to maintain the widget tree inside a Ui.
//   - Web/Desktop have different states?
//     - iced has a State type for widgets on both Web and Desktop.
//       - These are different though
//       - Desktop and Web provide two different widget states
//       - Could we have a `WidgetState` type which is generic.
//
// Reasons to keep Ui separate from renderer:
//
// - Tools like GTK Inspector could be built to render and app inside a popup for debugging.
//
// Against keeping Ui separate from renderer:
//
// - One implementation is easier to manage
// - cfg attributes could do the job but it's not satisfying and requires in tree approaches to add new platforms.

/// The main object of a UI.
///
/// A context creates an internal `Node` tree. The tree is incrementally updated as events are processed.
///
/// This type should not be used in public apis, instead the implementation should wrap this type with their
/// own `Context`.
pub struct Context<Data, Node> {
    /// The view tree.
    storage: Vec<Node>,
    /// The data of the context.
    data: Data,
}

impl<Data, Node> Context<Data, Node>
where
    Node: ViewNode<Data>,
{
    pub fn new(data: Data) -> Self {
        Self {
            storage: Vec::new(),
            data,
        }
    }

    /// Consumes the context, returning the data associated with the context.
    pub fn into_inner(self) -> Data {
        self.data
    }

    // TODO: Visitor
}

/// An object-safe view node which type erases a [`View`].
///
/// This trait may be extended to allow nodes to provide extra functionality, such as layout or accessibility
/// information.
pub trait ViewNode<T> {
    fn rebuild(&mut self);
}

struct Node<T, V: View<T>> {
    view: V,
    state: V::State,
}

impl<T, V> ViewNode<T> for Node<T, V>
where
    V: View<T> + Sized,
{
    fn rebuild(&mut self) {
        self.state = self.view.build();
    }
}
