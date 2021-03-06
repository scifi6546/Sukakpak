use super::*;
//Decisons: Go up Lift, Go Down run to new start
//To go up I need to know the end and start pos
//To Go down the start and end are supplied
//
#[derive(Clone, Debug)]
pub struct DecisionCost {
    pub cost: GraphWeight,
    pub end: GraphNode,
}
impl DecisionCost {
    pub fn inf() -> Self {
        Self {
            cost: GraphWeight::Infinity,
            end: GraphNode(Vector2::new(0, 0)),
        }
    }
}
pub trait Decision: Send {
    fn get_cost(
        &self,
        start: GraphNode,
        layers: &Vec<Mutex<Box<dyn GraphLayer>>>,
    ) -> (DecisionCost, Path);
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoToLift {
    start: GraphNode,
    end: GraphNode,
}
impl Decision for GoToLift {
    fn get_cost(
        &self,
        start: GraphNode,
        layers: &Vec<Mutex<Box<dyn GraphLayer>>>,
    ) -> (DecisionCost, Path) {
        let path = dijkstra(&start, &self.start, layers);
        (
            DecisionCost {
                cost: path.cost() + GraphWeight::Some(1),
                end: self.end,
            },
            path,
        )
    }
}
pub struct DecisionTree {
    root: Mutex<Box<dyn Decision>>,
    root_end: GraphNode,
    children: Vec<DecisionTree>,
}
impl DecisionTree {
    pub fn new(
        start: GraphNode,
        layers: &Vec<Mutex<Box<dyn GraphLayer>>>,
    ) -> (Self, DecisionCost, Path) {
        let mut out: Option<(Self, DecisionCost, Path)> = None;
        for decision in get_decisions(layers).drain(..) {
            let (tree, cost, path) = Self::build(start, decision, layers, 2);
            if out.is_some() {
                let (_out_tree, out_cost, out_path) = out.as_ref().unwrap();
                if out_cost.cost > cost.cost {
                    out = Some((tree, cost, out_path.clone()));
                }
            } else {
                out = Some((tree, cost, path));
            }
        }
        out.unwrap()
    }

    /// If recurse_levels hits zero recursion is stopped
    pub fn build(
        start: GraphNode,
        root: Box<dyn Decision>,
        layers: &Vec<Mutex<Box<dyn GraphLayer>>>,
        recurse_levels: u8,
    ) -> (Self, DecisionCost, Path) {
        let (root_cost, root_path) = root.get_cost(start, layers);
        let (children, lowest_child_cost, lowest_cost_path): (
            Vec<DecisionTree>,
            Option<DecisionCost>,
            Option<Path>,
        ) = if recurse_levels > 0 {
            let mut child = get_decisions(layers)
                .drain(..)
                .map(|dec| DecisionTree::build(root_cost.end, dec, layers, recurse_levels - 1))
                .collect::<Vec<_>>();
            let mut cost: Option<DecisionCost> = None;
            let mut lowest_cost_path: Option<Path> = None;

            for (_child, child_cost, child_path) in child.iter() {
                if let Some(c) = cost.clone() {
                    if child_cost.cost < c.cost {
                        lowest_cost_path = Some(child_path.clone());
                        cost = Some(child_cost.clone());
                    }
                } else {
                    lowest_cost_path = Some(child_path.clone());
                    cost = Some(child_cost.clone())
                }
            }
            (
                child.drain(..).map(|(c, _, _)| c).collect(),
                cost,
                lowest_cost_path,
            )
        } else {
            (vec![], None, None)
        };
        let (cost, path) = if let Some(child_cost) = lowest_child_cost {
            (
                DecisionCost {
                    cost: child_cost.cost + root_cost.cost,
                    end: child_cost.end,
                },
                root_path.append(lowest_cost_path.as_ref().unwrap()),
            )
        } else {
            (root_cost.clone(), root_path)
        };

        (
            Self {
                root: Mutex::new(root),
                root_end: root_cost.end,
                children,
            },
            cost,
            path,
        )
    }
}
///Gets possible decisions for a given
fn get_decisions(layers: &[Mutex<Box<dyn GraphLayer>>]) -> Vec<Box<dyn Decision>> {
    layers
        .iter()
        .filter(
            |layer| match layer.lock().expect("failed to get layer").get_type() {
                GraphType::Lift { .. } => true,
                _ => false,
            },
        )
        .map(|lift| match lift.lock().expect("").get_type() {
            GraphType::Lift { start, end } => {
                Box::new(GoToLift { start, end }) as Box<dyn Decision>
            }
            _ => panic!(),
        })
        .collect()
}
