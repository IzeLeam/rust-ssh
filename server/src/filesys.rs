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
        pub parent: Option<String>,
        pub children: Option<Vec<String>>,
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
                children.push(child.name.clone());
                child.parent = Some(self.name.clone());
            } else {
                panic!("Cannot add a child to a file node");
            }
        }

        

        pub fn pwd(&self) -> String {
            let mut path = String::from("/");
            let mut current_name = self.name.clone();

            while current_name != "root".to_string() {
                path = format!("{}/{}", current_name, path);
                if let Some(parent) = &self.parent {
                    current_name = parent.clone();
                } else {
                    break;
                }
            }

            path
        }

        pub fn ls(&self) -> Vec<String> {
            if let Some(children) = &self.children {
                children.iter().map(|child| child.clone()).collect()
            } else {
                vec![]
            }
        }

        pub fn cd(&self, name: &str) -> Option<String> {
            if name == ".." {
                if let Some(p) = &self.parent {
                    return Some(p.clone());
                } else {
                    return None;
                }
            }

            if let Some(childs) = &self.children {
                for c in childs {
                    if c == name  {
                        return Some(c.clone());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tree() {
        let root = filesys::create_tree();
        assert_eq!(root.name, "root");
        assert_eq!(root.node_type, filesys::NodeType::Directory);
        assert_eq!(root.ls(), vec!["dir1", "dir2", "dir3"]);
    }

    #[test]
    fn test_ls() {
        let root = filesys::create_tree();
        let dir1 = root.cd("dir1").unwrap();
        //assert_eq!(dir1.ls(), vec!["file1.txt", "file2.txt"]);
    }

    #[test]
    fn test_cd() {
        //let root = filesys::create_tree();
        //let dir1 = root.cd("dir1").unwrap();
        //let file1 = dir1.cd("file1.txt");
        //assert!(file1.is_none());
    }
}
