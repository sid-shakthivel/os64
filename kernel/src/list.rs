// src/list.rs

// Each node stores a reference to the next/previous node within the list along with a payload
#[derive(Debug)]
pub struct Node<T: 'static> {
    pub payload: T,
    pub next: Option<*mut Node<T>>,
    pub prev: Option<*mut Node<T>>,
}

// LIFO (Last in, First out)
#[derive(Debug)]
pub struct Stack<T: 'static> {
    pub head: Option<*mut Node<T>>,
    pub tail: Option<*mut Node<T>>,
    pub length: usize,
}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item =  Option<&'a Node<T>>;
    type IntoIter = StackIntoIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        StackIntoIterator {
            current: unsafe { Some(& *self.head.unwrap()) }
        }
    }
}
pub struct StackIntoIterator<'a, T: 'static> {
    current: Option<&'a Node<T>>
}

impl<'a, T> Iterator for StackIntoIterator<'a, T> {
    type Item =  Option<&'a Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let saved_current = self.current;
        if saved_current.is_none() { return None; }

        let next = self.current.unwrap().next;
        if next.is_some() { 
            self.current = unsafe { Some(& *next.unwrap()) } 
        } else {
            self.current = None;
        }

        return Some(saved_current);
    }
}

impl<T: Clone> Stack<T> {
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
            unsafe {
                // Update head to become next value and update the prev value
                self.head = (*self.head.unwrap()).next;
                (*self.head.unwrap()).prev = None;

                if self.head.is_none() {
                    self.tail = None;
                }
            }
        }

        self.length -= 1;
        head.expect("Attempted to pop from an empty item")
    }

    // TODO: Will remove at certain index
    pub fn remove_at(&mut self, index: usize) {
        if index > self.length { panic!("Index out of bounds") }; 
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
