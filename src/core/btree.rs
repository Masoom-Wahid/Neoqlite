use std::cmp::Ord;
use std::mem;

#[derive(Clone, Debug, Default)]
pub struct Node<K, V>
where
    K: Ord + Clone + Default,
    V: Clone + Default,
{
    pub keys: Vec<K>,
    pub values: Vec<V>,
    pub children: Vec<Box<Node<K, V>>>,
    pub is_leaf: bool,
    pub order: usize,
}

impl<K, V> Node<K, V>
where
    K: Ord + Clone + Default,
    V: Clone + Default,
{
    fn new(is_leaf: bool, order: usize) -> Self {
        Node {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            is_leaf,
            order,
        }
    }

    pub fn is_full(&self) -> bool {
        self.keys.len() == (self.order * 2 - 1) as usize
    }

    fn find_key(&self, key: &K) -> usize {
        let mut i = 0;
        while i < self.keys.len() && self.keys[i] < *key {
            i += 1;
        }
        i
    }

    fn borrow_from_prev(&mut self, idx: usize) {
        let mut sibling = mem::take(&mut self.children[idx - 1]);
        let child = &mut self.children[idx];

        let last_key = sibling.keys.pop().expect("Sibling has no keys to borrow");
        let last_value = sibling
            .values
            .pop()
            .expect("Sibling has no values to borrow");
        child.keys.insert(0, self.keys[idx - 1].clone());
        child.values.insert(0, self.values[idx - 1].clone());

        self.keys[idx - 1] = last_key;
        self.values[idx - 1] = last_value;
        if !sibling.is_leaf {
            if let Some(child_to_move) = sibling.children.pop() {
                child.children.insert(0, child_to_move);
            }
        }

        self.children[idx - 1] = sibling;
    }

    fn borrow_from_next(&mut self, idx: usize) {
        let mut sibling = mem::take(&mut self.children[idx + 1]);
        let child = &mut self.children[idx];

        let first_key = sibling.keys.remove(0);
        let first_value = sibling.values.remove(0);
        child.keys.push(self.keys[idx].clone());
        child.values.push(self.values[idx].clone());

        self.keys[idx] = first_key;
        self.values[idx] = first_value;

        if !sibling.is_leaf {
            child.children.push(sibling.children.remove(0));
        }

        self.children[idx + 1] = sibling;
    }

    fn merge(&mut self, idx: usize) {
        let mut sibling = self.children.remove(idx + 1);
        let child = &mut self.children[idx];

        child.keys.push(self.keys.remove(idx));
        child.values.push(self.values.remove(idx));

        child.keys.extend(sibling.keys.clone());
        child.values.extend(sibling.values.clone());

        if !child.is_leaf {
            child.children.extend(sibling.children.drain(..));
        }
    }

    fn fill(&mut self, idx: usize) {
        if idx > 0 && self.children[idx - 1].keys.len() >= self.order {
            self.borrow_from_prev(idx);
        } else if idx < self.children.len() - 1 && self.children[idx + 1].keys.len() >= self.order {
            self.borrow_from_next(idx);
        } else {
            if idx < self.children.len() - 1 {
                self.merge(idx);
            } else {
                self.merge(idx - 1);
            }
        }
    }

    fn delete_internal_node(&mut self, key: &K, idx: usize) {
        if self.children[idx].keys.len() >= self.order {
            let pred_key = self.get_predecessor_key(idx);
            let pred_value = self.get_predecessor_value(idx);
            self.keys[idx] = pred_key.clone();
            self.values[idx] = pred_value.clone();
            self.children[idx].delete(&pred_key);
        } else if self.children[idx + 1].keys.len() >= self.order {
            let succ_key = self.get_successor_key(idx);
            let succ_value = self.get_successor_value(idx);
            self.keys[idx] = succ_key.clone();
            self.values[idx] = succ_value.clone();
            self.children[idx + 1].delete(&succ_key);
        } else {
            self.merge(idx);
            self.children[idx].delete(key);
        }
    }

    fn get_predecessor_key(&self, idx: usize) -> K {
        let mut current = &self.children[idx];
        while !current.is_leaf {
            current = &current.children[current.children.len() - 1];
        }
        current.keys[current.keys.len() - 1].clone()
    }

    fn get_predecessor_value(&self, idx: usize) -> V {
        let mut current = &self.children[idx];
        while !current.is_leaf {
            current = &current.children[current.children.len() - 1];
        }
        current.values[current.values.len() - 1].clone()
    }

    fn get_successor_key(&self, idx: usize) -> K {
        let mut current = &self.children[idx + 1];
        while !current.is_leaf {
            current = &current.children[0];
        }
        current.keys[0].clone()
    }

    fn get_successor_value(&self, idx: usize) -> V {
        let mut current = &self.children[idx + 1];
        while !current.is_leaf {
            current = &current.children[0];
        }
        current.values[0].clone()
    }

    fn delete(&mut self, key: &K) {
        let mut idx = self.find_key(key);

        if idx < self.keys.len() && self.keys[idx] == *key {
            if self.is_leaf {
                self.keys.remove(idx);
                self.values.remove(idx);
            } else {
                self.delete_internal_node(key, idx);
            }
        } else {
            if self.is_leaf {
                return;
            }

            let flag = idx == self.keys.len();

            if self.children[idx].keys.len() < self.order {
                self.fill(idx);
            }

            if flag && idx > self.keys.len() {
                idx -= 1;
            }

            self.children[idx].delete(key);
        }
    }

    fn split_child(&mut self, i: usize) {
        let t = self.order;
        let child = self.children[i].as_mut();

        let mut new_child = Node::new(child.is_leaf, child.order);

        new_child.keys = child.keys.split_off(t);
        new_child.values = child.values.split_off(t);

        if !child.is_leaf {
            new_child.children = child.children.split_off(t);
        }

        self.keys
            .insert(i, child.keys.pop().expect("Child has no keys to move"));
        self.values
            .insert(i, child.values.pop().expect("Child has no values to move"));

        self.children.insert(i + 1, Box::new(new_child));
    }

    fn search(&self, key: &K) -> Option<&V> {
        let idx = self.find_key(key);

        if idx < self.keys.len() && self.keys[idx] == *key {
            return Some(&self.values[idx]);
        }

        if self.is_leaf {
            return None;
        }

        self.children[idx].search(key)
    }

    fn insert_non_full(&mut self, key: K, value: V) {
        let mut i = self.keys.len();

        if self.is_leaf {
            while i > 0 && key < self.keys[i - 1] {
                i -= 1;
            }

            self.keys.insert(i, key);
            self.values.insert(i, value);
        } else {
            while i > 0 && key < self.keys[i - 1] {
                i -= 1;
            }

            if self.children[i].is_full() {
                self.split_child(i);

                if key > self.keys[i] {
                    i += 1;
                }
            }

            self.children[i].insert_non_full(key, value);
        }
    }
}

#[derive(Debug)]
pub struct BTree<K, V>
where
    K: Ord + Clone + Default,
    V: Clone + Default,
{
    pub root: Option<Box<Node<K, V>>>,
    pub order: usize,
}

impl<K, V> BTree<K, V>
where
    K: Ord + Clone + Default + std::fmt::Debug,
    V: Clone + Default + std::fmt::Debug,
{
    pub fn new(order: usize) -> Self {
        assert!(order >= 2, "BTree order must be at least 2");
        BTree { root: None, order }
    }

    pub fn search(&self, key: &K) -> Option<&V> {
        self.root.as_ref().and_then(|r| r.search(key))
    }

    pub fn delete(&mut self, key: &K) {
        println!("got key of {:?}", key);
        if let Some(mut root_node) = self.root.take() {
            root_node.delete(key);

            if root_node.keys.is_empty() && !root_node.is_leaf {
                self.root = Some(root_node.children.remove(0));
            } else if root_node.keys.is_empty() && root_node.is_leaf {
                self.root = None;
            } else {
                self.root = Some(root_node);
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(ref mut root) = self.root {
            if root.is_full() {
                let mut new_root = Node::new(false, self.order);
                new_root.children.push(mem::take(root));
                new_root.split_child(0);

                let i = if new_root.keys[0] < key { 1 } else { 0 };
                new_root.children[i].insert_non_full(key, value);
                self.root = Some(Box::new(new_root));
            } else {
                root.insert_non_full(key, value);
            }
        } else {
            let mut root = Node::new(true, self.order);
            root.keys.push(key);
            root.values.push(value);
            self.root = Some(Box::new(root));
        }
    }

    pub fn values_in_order(&self) -> Vec<V> {
        let mut result = Vec::new();
        if let Some(ref root) = self.root {
            Self::collect_values_in_order(root, &mut result);
        }
        result
    }

    fn collect_values_in_order(node: &Node<K, V>, result: &mut Vec<V>) {
        if node.is_leaf {
            result.extend(node.values.iter().cloned());
        } else {
            for i in 0..node.keys.len() {
                Self::collect_values_in_order(&node.children[i], result);
                result.push(node.values[i].clone());
            }
            if !node.children.is_empty() {
                Self::collect_values_in_order(&node.children[node.children.len() - 1], result);
            }
        }
    }
}
