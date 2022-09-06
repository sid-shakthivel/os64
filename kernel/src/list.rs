// src/list.rs

use crate::{
    allocator::kmalloc,
    page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR},
};

// Each node stores a reference to the next/previous node within the list along with a payload
#[derive(Debug, Copy, Clone)]
pub struct Node<T: 'static> {
    pub payload: T,
    pub next: Option<*mut Node<T>>,
    pub prev: Option<*mut Node<T>>,
}

// LIFO (Last in, First out)
#[derive(Debug, Clone, PartialEq, Copy)]
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
        match next {
            Some(value) => self.current = unsafe { Some(&*value) },
            None => {
                self.current = None;
            }
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

    // Appends a new node to the start of the stack and allocates memory for it
    pub fn push(&mut self, value: T) {
        // let size = size_of::<Node<T>>();
        // let address = kmalloc(size as u64);
        let address = PAGE_FRAME_ALLOCATOR.lock().alloc_frame();
        PAGE_FRAME_ALLOCATOR.free();

        let new_node = Node::new(address as u64, value);

        match self.head {
            Some(head) => {
                unsafe {
                    // Update next of new node to head and head prev to new node
                    (*new_node).next = self.head;
                    (*head).prev = Some(new_node);
                }
            }
            None => {
                // Set tail to head too
                self.tail = Some(new_node);
            }
        }

        // Push to front of list
        self.head = Some(new_node);

        self.length += 1;
    }

    // Appends a new node to the start of the stack but stores the node at a specific address
    pub fn push_at_address(&mut self, address: u64, value: T) {
        let new_node = Node::new(address, value);

        match self.head {
            Some(head) => {
                unsafe {
                    // Update next of new node to head and head prev to new node
                    (*new_node).next = self.head;
                    (*head).prev = Some(new_node);
                }
            }
            None => {
                // Set tail to head too
                self.tail = Some(new_node);
            }
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
                // Update head to become next value and update the prev vlue if it's not None
                self.head = (*self.head.unwrap()).next;

                match self.head {
                    Some(head) => (*head).prev = None,
                    None => self.tail = None,
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

    pub fn get_at(&mut self, index: usize) -> T {
        for (i, node) in &mut self.into_iter().enumerate() {
            if i == index {
                return node.unwrap().payload.clone();
            }
        }

        panic!("Element not found");
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
        Returns stack of nodes in higher position then the target node (moves towards target from head)
        NOTE: Uses page frame allocator
    */
    pub fn get_higher_nodes(&mut self, target_node: T) -> Stack<T> {
        let mut new_stack = Stack::<T>::new();
        let mut length = 0;
        for (_i, node) in self.into_iter().enumerate() {
            if node.unwrap().payload.clone() == target_node {
                break;
            }
            new_stack.push(node.unwrap().payload.clone());
            length += 1;
        }
        new_stack.length = length;
        new_stack
    }

    /*
        Returns stack of nodes in a lower position then target node (moves from target towards tail)
        NOTE: Utilises page frame allocator
    */
    pub fn get_lower_nodes(&mut self, target_node: T) -> Stack<T> {
        let mut new_stack = Stack::<T>::new();
        let mut can_push = false;

        for (_i, node) in self.into_iter().enumerate() {
            if can_push {
                new_stack.push(node.unwrap().payload.clone());
            }

            if node.unwrap().payload.clone() == target_node {
                can_push = true;
            }
        }

        new_stack
    }

    // Appends another list onto this one
    pub fn append(&mut self, stack: Stack<T>) {
        // If head is none, completely append the new list
        if self.head.is_none() {
            self.head = stack.head;
            self.tail = stack.tail;
        } else {
            if let Some(stack_head) = stack.head {
                // Append the head onto the tail if it's actually full
                unsafe {
                    (*self.tail.unwrap()).next = Some(stack_head);
                    (*stack_head).prev = self.tail;
                }

                // Set the tail to the new stack's tail
                self.tail = stack.tail;
            }
        }

        // Update length
        self.length += stack.length;
    }

    // Removes every element from a list
    pub fn empty(&mut self) {
        while self.head.is_some() {
            let _address = self.pop() as *mut u64;
            // kfree(address);
        }
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
