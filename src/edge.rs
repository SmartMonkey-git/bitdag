/// A directed edge representing a parent-child relationship in a graph.
#[derive(Hash, Eq, Debug, Clone, PartialEq, Ord, PartialOrd)]
pub struct Edge {
    parent: String,
    child: String,
}

impl Edge {
    pub fn new(parent: String, child: String) -> Self {
        Self { parent, child }
    }

    pub fn parent(&self) -> &str {
        &self.parent
    }

    pub fn child(&self) -> &str {
        &self.child
    }

    pub fn parent_child(self) -> (String, String) {
        (self.parent, self.child)
    }
}

impl From<(String, String)> for Edge {
    fn from(value: (String, String)) -> Self {
        Edge::new(value.0, value.1)
    }
}

impl From<&(String, String)> for Edge {
    fn from(value: &(String, String)) -> Self {
        Edge::new(value.0.clone(), value.1.clone())
    }
}

impl From<(&str, &str)> for Edge {
    fn from(value: (&str, &str)) -> Self {
        Edge::new(value.0.to_string(), value.1.to_string())
    }
}
