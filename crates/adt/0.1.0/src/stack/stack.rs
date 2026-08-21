/*
 * stack.rs
 * Defines a class that represents a stack
 * Created by Andrew Davis
 * Created on 1/15/2019
 * Licensed under the Lesser GNU Public License, version 3
 */

//use statements
use std::collections::LinkedList;

/// A FIFO data structure
pub struct Stack<T> {
    /// The elements that make up the Stack 
    elems: LinkedList<T>,
}

//structure implementation
impl<T> Stack<T> {
    /// Returns a new Stack object
    pub fn new() -> Self {
        return Stack {
            elems: LinkedList::new()
        };
    }

    /// Pushes a value onto the Stack
    ///
    /// # Arguments
    ///
    /// * `value` - The value to push onto the Stack 
    pub fn push(&mut self, value: T) {
        self.elems.push_back(value); 
    }

    /// Pops a value off the Stack and returns it as an Option
    pub fn pop(&mut self) -> Option<T> {
        return self.elems.pop_back();
    }

    /// Returns an immutable reference to the value on top of the Stack
    pub fn peek(&self) -> Option<&T> {
        return self.elems.back();
    }

    /// Returns a mutable reference to the value on top of the Stack
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        return self.elems.back_mut();
    }

    /// Returns the number of items on the Stack
    pub fn size(&self) -> usize {
        return self.elems.len();
    }

    /// Returns whether the Stack is empty
    pub fn is_empty(&self) -> bool {
        return self.elems.is_empty();
    }

    /// Clears the Stack
    pub fn clear(&mut self) {
        self.elems.clear();
    }
}

//end of file
