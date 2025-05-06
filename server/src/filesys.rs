pub mod filesys {

    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    #[derive(Debug, Clone, PartialEq)]
    pub enum NodeType {
        File,
        Directory,
    }

    #[derive(Debug, Clone)]
    pub struct Node {
        pub name: String,
        pub node_type: NodeType,
        pub parent: Option<Weak<RefCell<Node>>>,
        pub children: Option<Vec<Rc<RefCell<Node>>>>,
    }

    impl Node {
        pub fn new_file(name: String) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Node {
                name,
                node_type: NodeType::File,
                parent: None,
                children: None,
            }))
        }

        pub fn new_directory(name: String) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Node {
                name,
                node_type: NodeType::Directory,
                parent: None,
                children: Some(vec![]),
            }))
        }

        pub fn add_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
            {
                let mut child_mut = child.borrow_mut();
                child_mut.parent = Some(Rc::downgrade(parent));
            }
    
            let mut parent_mut = parent.borrow_mut();
            if parent_mut.node_type == NodeType::Directory {
                if let Some(children) = &mut parent_mut.children {
                    children.push(child);
                }
            } else {
                panic!("Cannot add a child to a file node");
            }
        }

        

        
        pub fn pwd(node: Rc<RefCell<Node>>) -> String {
            let mut path = String::new();
            let mut current_node = Some(node);
    
            while let Some(rc_node) = current_node {
                let borrowed = rc_node.borrow();
                path = format!("/{}", borrowed.name) + &path;
                current_node = borrowed
                    .parent
                    .as_ref()
                    .and_then(|weak_parent| weak_parent.upgrade());
            }
    
            if path.is_empty() {
                "/".to_string()
            } else {
                path
            }
        }

        // Probleme avec ls avec la suite de commande suivante : 
        // cd dir1 ; cd .. ; ls => La liste des enfants est vide donc le prog bloque 
        pub fn ls(&self) -> Vec<String> {
            if self.node_type == NodeType::Directory {
                if let Some(children) = &self.children {
                    children.iter()
                        .map(|child| child.borrow().name.clone())
                        .collect()
                } else {
                    vec![]
                }
            } else {
                println!("This node is not a directory");
                vec![]
            }
        }
        
        pub fn cd(current: &Rc<RefCell<Node>>, name: &str) -> Option<Rc<RefCell<Node>>> {
            let current_borrowed = current.borrow();
    
            if name == ".." {
                if let Some(ref weak_parent) = current_borrowed.parent {
                    if let Some(parent_rc) = weak_parent.upgrade() {
                        println!("Parent directory: {:?}", parent_rc.borrow().name);
                        return Some(parent_rc);
                    } else {
                        println!("Parent has been dropped");
                        return None;
                    }
                } else {
                    println!("No parent directory");
                    return None;
                }
            }
    
            if let Some(children) = &current_borrowed.children {
                for child in children {
                    let child_borrowed = child.borrow();
                    if child_borrowed.name == name && child_borrowed.node_type == NodeType::Directory {
                        return Some(Rc::clone(child));
                    }
                }
            }
    
            println!("Directory '{}' not found", name);
            None
        }

        pub fn tab_complete_arg(&self, start: &str) -> Vec<String> {
            if let Some(children) = &self.children {
                let mut completions = Vec::new();
                for child in children {
                    if child.borrow().name.starts_with(start) {
                        completions.push(child.borrow().name.clone());
                    }
                }
                completions
            } else {
                vec![]
            }
        }

        pub fn tab_complete(&self, start: &str) -> Vec<String> {
            let mut completions = Vec::new();
            if let Some(children) = &self.children {
                for child in children {
                    if child.borrow().name.starts_with(start) {
                        completions.push(child.borrow().name.clone());
                    }
                }
            }
            completions
        }
    }

    pub fn create_tree() -> Rc<RefCell<Node>> {
        let root = Node::new_directory("root".to_string());
    
        let dir1 = Node::new_directory("dir1".to_string());
        let dir2 = Node::new_directory("dir2".to_string());
        let dir3 = Node::new_directory("dir3".to_string());
    
        let file1 = Node::new_file("file1.txt".to_string());
        let file2 = Node::new_file("file2.txt".to_string());
        let file3 = Node::new_file("file3.txt".to_string());
        let file4 = Node::new_file("file4.txt".to_string());
    
        Node::add_child(&dir1, file1);
        Node::add_child(&dir1, file2);
        Node::add_child(&dir2, file3);
        Node::add_child(&dir3, file4);
    
        Node::add_child(&root, dir1);
        Node::add_child(&root, dir2);
        Node::add_child(&root, dir3);
    
        root
    }
}
