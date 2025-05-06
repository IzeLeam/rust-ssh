pub mod filesys {
    #[derive(Debug, Clone, PartialEq)]
    pub enum NodeType {
        File,
        Directory,
    }

    #[derive(Debug, Clone)]
    pub struct Node {
        pub name: String,
        pub node_type: NodeType,
        pub parent: Option<Box<Node>>,
        pub children: Option<Vec<Node>>,
    }

    impl Node {
        pub fn new_file(name: String) -> Self {
            Node {
                name,
                node_type: NodeType::File,
                parent: None,
                children: None,
            }
        }

        pub fn new_directory(name: String) -> Self {
            Node {
                name,
                node_type: NodeType::Directory,
                parent: None,
                children: Some(Vec::new()),
            }
        }

        pub fn add_child(&mut self, child: &mut Node) {
            if let Some(children) = &mut self.children {
                children.push(child.clone());
                child.parent = Some(Box::new(self.clone()));
            } else {
                panic!("Cannot add a child to a file node");
            }
        }

        pub fn _get_children(&self) -> Option<&Vec<Node>> {
            self.children.as_ref()
        }

        pub fn pwd(&self) -> String {
            let mut path = String::new();
            let mut current_node = self;

            while let Some(parent) = &current_node.parent {
                println!("Current node: {:?}", current_node.name);
                path = format!("/{}", current_node.name) + &path;
                current_node = parent;
            }
            path
        }

        pub fn ls(&self) -> Vec<String> {
            if let Some(children) = &self.children {
                children.iter().map(|child| child.name.clone()).collect()
            } else {
                vec![]
            }
        }

        pub fn cd(&self, name: &str) -> Option<Node> {
            if name == ".." {
                if let Some(parent) = &self.parent {
                    return Some(*parent.clone());
                } else {
                    return None;
                }
            }

            if let Some(children) = &self.children {
                for child in children {
                    if child.name == name  && child.node_type == NodeType::Directory {
                        return Some(child.clone());
                    }
                }
            }
            None
        }
    }

    pub fn create_tree() -> Node {
        let mut root = Node::new_directory("root".to_string());
        let mut dir1 = Node::new_directory("dir1".to_string());
        let mut dir2 = Node::new_directory("dir2".to_string());
        let mut dir3 = Node::new_directory("dir3".to_string());

        let mut file1 = Node::new_file("file1.txt".to_string());
        let mut file2 = Node::new_file("file2.txt".to_string());
        let mut file3 = Node::new_file("file3.txt".to_string());
        let mut file4 = Node::new_file("file4.txt".to_string());

        dir1.add_child(&mut file1);
        dir1.add_child(&mut file2);
        dir2.add_child(&mut file3);
        dir3.add_child(&mut file4);

        root.add_child(&mut dir1);
        root.add_child(&mut dir2);
        root.add_child(&mut dir3);

        root
    }

}
