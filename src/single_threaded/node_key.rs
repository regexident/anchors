use std::{marker::PhantomData, rc::Rc};

use super::node::NodePtr;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(super) struct NodeKey {
    pub(super) ptr: NodePtr,
    pub(super) token: u32,
    // Make type !Send + !Sync:
    _phantom: PhantomData<Rc<()>>,
}

impl NodeKey {
    pub(super) fn new(ptr: NodePtr, token: u32) -> Self {
        Self {
            ptr,
            token,
            _phantom: PhantomData,
        }
    }
}
