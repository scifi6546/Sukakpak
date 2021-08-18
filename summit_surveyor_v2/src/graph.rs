use priority_queue::PriorityQueue;
use std::{cmp::Reverse, collections::HashMap, sync::Mutex};
pub use sukakpak::nalgebra::Vector2;
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub struct GraphNode(pub Vector2<usize>);
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum GraphWeight {
    Some(i32),
    Infinity,
}
impl GraphWeight {
    pub fn is_finite(&self) -> bool {
        match self {
            GraphWeight::Some(_) => true,
            GraphWeight::Infinity => false,
        }
    }
}
impl std::fmt::Display for GraphWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Infinity => write!(f, "Infinity"),
            Self::Some(v) => write!(f, "Some({})", v),
        }
    }
}
impl std::ops::Add for GraphWeight {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match self {
            Self::Some(num) => match other {
                Self::Some(other_num) => Self::Some(num + other_num),
                Self::Infinity => Self::Infinity,
            },
            Self::Infinity => Self::Infinity,
        }
    }
}
impl std::cmp::PartialOrd for GraphWeight {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for GraphWeight {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            Self::Infinity => match other {
                Self::Infinity => std::cmp::Ordering::Equal,
                Self::Some(_) => std::cmp::Ordering::Greater,
            },
            Self::Some(s) => match other {
                Self::Infinity => std::cmp::Ordering::Less,
                Self::Some(o) => s.cmp(o),
            },
        }
    }
}
impl std::iter::Sum for GraphWeight {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(GraphWeight::Some(0), |acc, x| acc + x)
    }
}
pub enum GraphType {
    Terrain,
    Lift { start: GraphNode, end: GraphNode },
}
pub trait GraphLayer: Send {
    /// gets the type of Graph
    fn get_type(&self) -> GraphType;
    /// Gets nodes connected to a node on a graph
    fn get_children(&self, point: &GraphNode) -> Vec<(GraphNode, GraphWeight)>;
    /// gets weight connecting two points. If points are not connecte infinity is
    /// returned
    fn get_distance(&self, start_point: &GraphNode, end_point: &GraphNode) -> GraphWeight;
}
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub path: Vec<(GraphNode, GraphWeight)>,
}
impl Path {
    pub fn new(path: Vec<(GraphNode, GraphWeight)>) -> Self {
        Self { path }
    }
    pub fn append(self, other: &Self) -> Self {
        let mut path = vec![];
        for p in self.path.iter() {
            path.push(p.clone());
        }
        for p in other.path.iter() {
            path.push(p.clone());
        }
        Self { path }
    }
    pub fn len(&self) -> usize {
        self.path.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn endpoint(&self) -> Option<&GraphNode> {
        if !self.path.is_empty() {
            Some(&self.path[self.path.len() - 1].0)
        } else {
            None
        }
    }
}
impl Default for Path {
    fn default() -> Self {
        Path { path: vec![] }
    }
}
impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for p in self.path.iter() {
            writeln!(f, "\t<{}, {}>: {}", p.0 .0.x, p.0 .0.y, p.1)?;
        }
        writeln!(f, "}}")
    }
}
/// Implements Dijkstra's algorythm on a generic graph.
/// used [wikipedia](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm) as a refrence
/// # Preconditions:
/// Graph Weights are greater than zero. If any of the graph weights are less then zero then
/// the alorythm panics
pub fn dijkstra(
    source: &GraphNode,
    destination: &GraphNode,
    graph: &[Mutex<Box<dyn GraphLayer>>],
) -> Path {
    //queue used to priortize searching
    let mut queue = PriorityQueue::new();
    //annotates previous node in shortest path tree. If item is not preseant then previous is marked as infinite.
    let mut previous = HashMap::new();
    //annotates the distance of the node from the source to a given node. If Node is not present then distance can be considered as infinite
    let mut distance = HashMap::<GraphNode, GraphWeight>::new();
    //inserting first node
    queue.push(source.clone(), Reverse(GraphWeight::Some(0)));
    distance.insert(source.clone(), GraphWeight::Some(0));
    while !queue.is_empty() {
        let (best_vertex, parent_distance) = queue.pop().unwrap();
        //getting neighbors
        for (child, child_distance) in graph.iter().flat_map(|g| {
            g.lock()
                .expect("failed to get lock")
                .get_children(&best_vertex)
        }) {
            assert!(child_distance >= GraphWeight::Some(0));
            let total_distance = child_distance.clone() + parent_distance.0.clone();
            let is_shortest_path = {
                if let Some(best_known_distance) = distance.get(&child) {
                    &total_distance < best_known_distance
                } else {
                    true
                }
            };
            if is_shortest_path {
                distance.insert(child.clone(), total_distance.clone());
                previous.insert(child.clone(), (best_vertex.clone(), child_distance.clone()));

                queue.push(child.clone(), Reverse(total_distance));
            }
        }
    }
    let mut path: Vec<(GraphNode, GraphWeight)> = vec![];
    let mut current = (destination.clone(), GraphWeight::Some(0));
    path.push(current.clone());
    loop {
        if let Some((node, weight)) = previous.get(&current.0) {
            path.push((node.clone(), weight.clone().clone()));
            current = (node.clone(), weight.clone().clone());
        } else {
            return Path {
                path: path.iter().rev().cloned().collect(),
            };
        }
    }
}
