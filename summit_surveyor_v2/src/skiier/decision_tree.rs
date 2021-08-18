use super::*;
pub trait Decision {
    fn get_cost(&self, end: GraphNode) -> GraphWeight;
}
//Decisons: Go up Lift, Go Down run to new start
//To go up I need to know the end and start pos
//To Go down the start and end are supplied
