use std::{marker::PhantomData, rc::Rc};

use super::node::NodePtr;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct NodeKey {
    pub(super) ptr: NodePtr,
    pub(super) token: u32,
    // Make type !Send + !Sync:
    _phantom: PhantomData<Rc<()>>,
}

impl NodeKey {
    pub fn new(ptr: NodePtr, token: u32) -> Self {
        Self {
            ptr,
            token,
            _phantom: PhantomData,
        }
    }
}
