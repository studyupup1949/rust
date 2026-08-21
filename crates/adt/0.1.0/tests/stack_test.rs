/*
 * stack_test.rs
 * Defines integrated tests for the Stack class
 * Created by Andrew Davis
 * Created on 1/15/2019
 * Licensed under the Lesser GNU Public License, version 3
 */

//import the adt crate
extern crate adt;

//use the Stack class
use adt::Stack;

// start of tests

// this function tests size checking of a Stack
#[test]
fn stack_size() {
    let stack: Stack<i32> = Stack::new();
    assert_eq!(stack.size(), 0);
}

//this function tests adding items to a Stack
#[test]
fn stack_push() {
    let mut stack: Stack<i32> = Stack::new();
    stack.push(42);
    assert_eq!(stack.size(), 1);
    stack.push(216);
    assert_eq!(stack.size(), 2);
}

//this function tests removing items from a Stack
#[test]
fn stack_pop() {
    let mut stack: Stack<i32> = Stack::new();
    stack.push(42);
    assert_eq!(stack.size(), 1);
    let off = stack.pop();
    assert_eq!(off, Some(42));
    assert_eq!(stack.size(), 0);
}

//this function tests peeking at the top of a Stack
#[test]
fn stack_peek() {
    let mut stack: Stack<i32> = Stack::new();
    stack.push(13);
    let top = stack.peek();
    assert_eq!(top, Some(&13));
}

//this function tests checking whether a Stack is empty
#[test]
fn stack_is_empty() {
    let mut stack: Stack<i32> = Stack::new();
    stack.push(216);
    assert_eq!(stack.is_empty(), false);
    stack.pop();
    assert_eq!(stack.is_empty(), true);
}

//this function tests clearing a Stack
#[test]
fn stack_clear() {
    let mut stack: Stack<i32> = Stack::new();
    stack.push(13);
    stack.push(42);
    stack.push(216);
    assert_eq!(stack.is_empty(), false);
    stack.clear();
    assert_eq!(stack.is_empty(), true);
}

//end of tests
