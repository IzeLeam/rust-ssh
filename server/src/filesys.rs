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
                child.parent = Some(self.name.clone());
            } else {
                panic!("Cannot add a child to a file node");
            }
        }

        fn find_node(&self, name: &str) -> Option<Node> {
            if self.name == name {
                return Some(self.clone());
            }

            if let Some(children) = &self.children {
                for child in children {
                    if let Some(node) = child.find_node(name) {
                        return Some(node);
                    }
                }
            }
            None
        }

        pub fn pwd(&self, current: &str) -> String {
            let mut path = String::from("/");
            let mut current_name = current.to_string();

            while current_name != "root".to_string() {
                path = format!("{}/{}", path, current_name);
                if let Some(parent) = &self.parent {
                    current_name = parent.clone();
                } else {
                    break;
                }
            }

            return path;
        }

        pub fn ls(&self, current: &str) -> Vec<String> {
            let n = self.find_node(current);
            if let Some(node) = n {
                if let Some(children) = &node.children {
                    let mut names = Vec::new();
                    for child in children {
                        names.push(child.name.clone());
                    }
                    return names;
                }
            }
            vec![]
        }

        pub fn cd(&self, current: &str, name: &str) -> Option<String> {
            if name == ".." {
                if let Some(p) = &self.parent {
                    return Some(p.clone());
                } else {
                    return None;
                }
            }

            let n = self.find_node(current);
            if let Some(node) = n {
                if let Some(children) = &node.children {
                    for child in children {
                        if child.name == name && child.node_type == NodeType::Directory {
                            return Some(child.name.clone());
                        }
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
    fn test_ls() {
        let root = filesys::create_tree();
        let current = String::from("root");
        assert_eq!(root.name, "root");
        assert_eq!(root.node_type, filesys::NodeType::Directory);
        assert_eq!(root.ls(&current), vec!["dir1", "dir2", "dir3"]);
    }

    #[test]
    fn test_cd() {
        let root = filesys::create_tree();
        let current = root.cd("root", "dir1").unwrap();
        assert_eq!(current, String::from("dir1"));
    }

    #[test]
    fn test_cd_to_parent() {
        let root = filesys::create_tree();
        let current = root.cd("dir1", "..").unwrap();
        assert_eq!(current, String::from("root"));
    }

    #[test]
    fn test_pwd() {
        let root = filesys::create_tree();
        let current = String::from("dir1");
        assert_eq!(root.pwd(&current), String::from("/dir1"));
    }
}
