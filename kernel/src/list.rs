// src/list.rs

use crate::{
    page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR},
    print_serial, CONSOLE,
};

// Each node stores a reference to the next/previous node within the list along with a payload
#[derive(Debug)]
pub struct Node<T: 'static> {
    pub payload: T,
    pub next: Option<*mut Node<T>>,
    pub prev: Option<*mut Node<T>>,
}

// LIFO (Last in, First out)
#[derive(Debug, Clone, PartialEq)]
pub struct Stack<T: 'static> {
    pub head: Option<*mut Node<T>>,
    pub tail: Option<*mut Node<T>>,
    pub length: usize,
}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = Option<&'a Node<T>>;
    type IntoIter = StackIntoIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        StackIntoIterator {
            current: match self.head {
                Some(head) => unsafe { Some(&*head) },
                _ => None,
            },
        }
    }
}
pub struct StackIntoIterator<'a, T: 'static> {
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for StackIntoIterator<'a, T> {
    type Item = Option<&'a Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let saved_current = self.current;
        if saved_current.is_none() {
            return None;
        }

        let next = self.current.unwrap().next;
        if next.is_some() {
            self.current = unsafe { Some(&*next.unwrap()) }
        } else {
            self.current = None;
        }

        return Some(saved_current);
    }
}

impl<T: Clone + core::cmp::PartialEq + core::fmt::Debug> Stack<T> {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            length: 0,
        }
    }

    // Appends a new node to the start of the stack
    pub fn push(&mut self, address: u64, value: T) {
        let new_node = Node::new(address, value);

        if self.head.is_some() {
            unsafe {
                // Update next of new node to head and head prev to new node
                (*new_node).next = self.head;
                (*self.head.unwrap()).prev = Some(new_node);
            }
        } else {
            // Set tail
            self.tail = Some(new_node);
        }

        // Push to front of list
        self.head = Some(new_node);

        self.length += 1;
    }

    // Pops a node from the start of the stack
    pub fn pop(&mut self) -> *mut Node<T> {
        let head = self.head.clone();
        if self.head.is_some() {
            self.length -= 1;

            unsafe {
                // Update head to become next value and update the prev value if it's not None
                self.head = (*self.head.unwrap()).next;
                if self.head.is_some() {
                    (*self.head.unwrap()).prev = None;
                }

                if self.head.is_none() {
                    self.tail = None;
                }
            }
        }

        head.expect("Attempted to pop from an empty item")
    }

    // Removes a node from linked list given position within list
    pub fn remove_at(&mut self, index: usize) -> *mut Node<T> {
        if index > self.length {
            panic!("Index out of bounds at {}\n", index);
        };

        if index == (self.length - 1) {
            return self.pop_tail();
        }

        match index {
            0 => self.pop(),
            _ => {
                self.length -= 1;
                let node = self.into_iter().nth(index).unwrap().unwrap();
                unsafe {
                    (*node.prev.unwrap()).next = node.next;
                    (*node.next.unwrap()).prev = node.prev;

                    let const_ptr = node as *const Node<T>;
                    const_ptr as *mut Node<T>
                }
            }
        }
    }

    // Removes a node from linked list given value
    pub fn remove(&mut self, target_node: &Node<T>) -> *mut Node<T> {
        let raw_value = target_node.payload.clone();
        for (i, node) in &mut self.into_iter().enumerate() {
            if node.unwrap().payload.clone() == raw_value {
                return self.remove_at(i);
            }
        }
        panic!("Cannot find element");
    }

    /*
        Returns stack of nodes in higher position then the target node
        NOTE: Uses page frame allocator
    */
    pub fn get_higher_nodes(&mut self, target_node: T) -> Stack<T> {
        let mut new_stack = Stack::<T>::new();
        let mut length = 0;
        for (i, node) in self.into_iter().enumerate() {
            if node.unwrap().payload.clone() == target_node {
                break;
            }
            new_stack.push(
                PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as u64,
                node.unwrap().payload.clone(),
            );
            PAGE_FRAME_ALLOCATOR.free();
            length += 1;
        }
        new_stack.length = length;
        new_stack
    }

    // Appends another list onto this one
    pub fn append(&mut self, stack: Stack<T>) {
        // Replace the tail with the head of the new stack
        unsafe {
            (*self.tail.unwrap()).next = stack.head;
        }

        // Set the tail to the new stack's tail
        self.tail = stack.tail;

        // Update length
        self.length += stack.length;
    }

    // Removes every element from a list
    pub fn empty(&mut self) {
        for i in 0..(self.length - 1) {
            self.remove_at(i);
        }
        self.length = 0;
    }

    // Removes the last element from the list
    fn pop_tail(&mut self) -> *mut Node<T> {
        unsafe {
            let clone = self.tail;
            if self.head == self.tail {
                // If only 1 node, make both null
                self.tail = None;
                self.head = None;
            } else {
                // Make second to last node the new tail and give it a next of None
                let new_tail = (*self.tail.unwrap()).prev;
                (*new_tail.unwrap()).next = None;
                self.tail = new_tail;
            }
            self.length -= 1;
            return clone.unwrap();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl<T: Clone> Node<T> {
    pub fn new(address: u64, payload: T) -> *mut Node<T> {
        unsafe {
            *(address as *mut Node<T>) = Node {
                payload,
                next: None,
                prev: None,
            };
        }
        address as *mut Node<T>
    }
}
