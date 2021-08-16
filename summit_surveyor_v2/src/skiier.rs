use super::prelude::GraphLayer;
use legion::*;
use std::sync::Mutex;
pub struct Skiier {}
#[system(for_each)]
pub fn skiier(skiier: &Skiier, #[resource] graph_layers: &Vec<Mutex<Box<dyn GraphLayer>>>) {}
