use std::fmt::Debug;

#[derive(Debug)]
pub struct List<T> {
    head: Option<Box<Node<T>>>,
    tail: Option<*mut Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> List<T>
where
    T: PartialEq<T>,
    T: Debug,
{
    pub fn new() -> Self {
        return List {
            tail: None,
            head: None,
        };
    }

    pub fn push(&mut self, item: T) {
        let new_node = Box::new(Node {
            value: item,
            next: None,
        });

        let raw_node = Box::into_raw(new_node);

        if let Some(tail) = self.tail {
            unsafe {
                (*tail).next = Some(Box::from_raw(raw_node));
            }
        } else {
            self.head = unsafe { Some(Box::from_raw(raw_node)) }
        }

        self.tail = Some(raw_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        return self.head.take().map(|head| {
            self.head = head.next;

            if self.head.is_none() {
                self.tail = None;
            }

            head.value
        });
    }

    pub fn len(&self) -> usize {
        let mut counter = 0;
        let mut current_node = &self.head;

        while let Some(node) = current_node {
            counter += 1;
            current_node = &node.next;
        }

        return counter;
    }

    pub fn is_empty(&self) -> bool {
        return self.head.is_none();
    }

    pub fn index(&self, value: T) -> Option<usize> {
        let mut current_node = &self.head;
        let mut current_index = 0;

        while let Some(node) = current_node {
            if node.value == value {
                return Some(current_index);
            } else {
                current_index += 1;
                current_node = &node.next;
            }
        }

        return None;
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            let mut current_node = &self.head;
            let mut current_index = 0;

            while let Some(node) = current_node {
                if current_index == index {
                    return Some(&node.value);
                } else {
                    current_node = &node.next;
                    current_index += 1;
                }
            }
        }

        return None;
    }

    pub fn insert(&mut self, value: T, index: usize) -> bool {
        if index == 0 {
            self.push(value);

            return true;
        }

        if index < self.len() {
            let mut current_node = &mut self.head;
            let mut current_index = 0;

            while current_index < index - 1 {
                if let Some(node) = current_node {
                    current_node = &mut node.next;
                    current_index += 1;
                }
            }

            if let Some(node) = current_node {
                let new_boxed_node = Box::new(Node {
                    value,
                    next: node.next.take(),
                });

                node.next = Some(new_boxed_node);

                return true;
            }
        }

        return false;
    }
}