use super::*;
struct DecisionCost {
    cost: GraphWeight,
    endpoint: GraphNode,
}
trait Decision {
    fn get_cost(&self, start: GraphNode) -> DecisionCost;
}
//Decisons: Go up Lift, Go Down run to new start
//To go up I need to know the end and start pos
//To Go down the start and end are supplied
// Two Types of decisions, path to go down (skiing, lifts) and destinations (lift bottoms,shops etc)
// Decisons also have costs
