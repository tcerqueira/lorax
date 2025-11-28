use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

#[derive(Debug)]
pub struct SharedNode<T>(Rc<Node<T>>);

#[derive(Debug, Clone)]
pub struct Node<T> {
    value: RefCell<T>,
    parent: Option<SharedNode<T>>,
}

#[derive(Debug, Clone)]
pub struct Chain<T> {
    end: Option<SharedNode<T>>,
}

impl<T> Chain<T> {
    pub fn new() -> Self {
        Self { end: None }
    }

    pub fn push(&mut self, value: T) {
        let old_end = self.end.take();
        let new_node = SharedNode::new(value, old_end);
        self.end = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<SharedNode<T>> {
        let new_end = self.end.as_ref().and_then(|n| n.0.parent.clone());
        std::mem::replace(&mut self.end, new_end)
    }

    pub fn head(&self) -> Option<RefMut<'_, T>> {
        self.end.as_ref().map(|n| n.value())
    }

    pub fn head_node(&self) -> Option<&SharedNode<T>> {
        self.end.as_ref()
    }

    pub fn iter(&self) -> ChainIter<'_, T> {
        ChainIter {
            current: self.end.as_ref(),
        }
    }
}

impl<T> SharedNode<T> {
    pub fn new(value: T, parent: Option<SharedNode<T>>) -> Self {
        Self(Rc::new(Node {
            value: RefCell::new(value),
            parent,
        }))
    }

    pub fn value(&self) -> RefMut<'_, T> {
        self.0.value.borrow_mut()
    }
}

pub struct ChainIter<'a, T> {
    current: Option<&'a SharedNode<T>>,
}

impl<'a, T> Iterator for ChainIter<'a, T> {
    type Item = RefMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = node.0.parent.as_ref();
        Some(node.value())
    }
}

impl<T> Clone for SharedNode<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
