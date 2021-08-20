use super::*;
//Decisons: Go up Lift, Go Down run to new start
//To go up I need to know the end and start pos
//To Go down the start and end are supplied
//
#[derive(Clone, Debug)]
pub struct DecisionCost {
    pub cost: GraphWeight,
    pub end: GraphNode,
    pub path: Path,
}
impl DecisionCost {
    pub fn inf() -> Self {
        Self {
            cost: GraphWeight::Infinity,
            end: GraphNode(Vector2::new(0, 0)),
            path: Default::default(),
        }
    }
}
pub trait Decision: Send {
    fn get_cost(&self, start: GraphNode, layers: &Vec<Mutex<Box<dyn GraphLayer>>>) -> DecisionCost;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoToLift {
    start: GraphNode,
    end: GraphNode,
}
impl Decision for GoToLift {
    fn get_cost(&self, start: GraphNode, layers: &Vec<Mutex<Box<dyn GraphLayer>>>) -> DecisionCost {
        let path = dijkstra(&start, &self.start, layers);
        DecisionCost {
            cost: path.cost() + GraphWeight::Some(1),
            path,
            end: self.end,
        }
    }
}
pub struct DecisionTree {
    root: Mutex<Box<dyn Decision>>,
    children: Vec<DecisionTree>,
}
impl DecisionTree {
    pub fn new(start: GraphNode, layers: &Vec<Mutex<Box<dyn GraphLayer>>>) -> (Self, DecisionCost) {
        let mut out: Option<(Self, DecisionCost)> = None;
        for decision in get_decisions(layers).drain(..) {
            let (tree, cost) = Self::build(start, decision, layers, 2);
            if out.is_some() {
                let (out_tree, out_cost) = out.as_ref().unwrap();
                if out_cost.cost > cost.cost {
                    out = Some((tree, cost));
                }
            } else {
                out = Some((tree, cost));
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
    ) -> (Self, DecisionCost) {
        let root_cost = root.get_cost(start, layers);
        let (children, cost): (_, Option<DecisionCost>) = if recurse_levels > 0 {
            let mut child = get_decisions(layers)
                .drain(..)
                .map(|dec| DecisionTree::build(root_cost.end, dec, layers, recurse_levels - 1))
                .collect::<Vec<_>>();
            let mut cost: Option<DecisionCost> = None;

            for (_child, child_cost) in child.iter() {
                if let Some(c) = cost.clone() {
                    if child_cost.cost < c.cost {
                        cost = Some(child_cost.clone());
                    }
                } else {
                    cost = Some(child_cost.clone())
                }
            }
            (child.drain(..).map(|(c, _)| c).collect(), cost)
        } else {
            (vec![], None)
        };
        let cost = if let Some(child_cost) = cost {
            DecisionCost {
                cost: child_cost.cost + root_cost.cost,
                path: child_cost.path,
                end: child_cost.end,
            }
        } else {
            root_cost
        };

        (
            Self {
                root: Mutex::new(root),
                children,
            },
            cost,
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
