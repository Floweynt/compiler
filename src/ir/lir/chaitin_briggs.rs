//register allocator implementation of Briggs et al. (1994)
//improvement of Chaitin et al. (1981)

#![allow(non_camel_case_types)]
use std::collections::{HashMap, HashSet};
use slotmap::{SlotMap, SecondaryMap, Key};
use crate::ir::lir::code::{LirGraph, NodeRef, NodeKind, virtual_register, spill_slot};

//for indexing blocks, prevents type mismatch with nodes
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct block_reference(slotmap::KeyData);

#[derive(Debug)]
pub enum coloring_result {
    success(HashMap<virtual_register, usize>),
    failure(Vec<virtual_register>),
}

unsafe impl slotmap::Key for block_reference {
    fn data(&self) -> slotmap::KeyData {
        self.0
    }
}

impl From<slotmap::KeyData> for block_reference {
    fn from(key: slotmap::KeyData) -> Self {
        block_reference(key)
    }
}

#[derive(Debug)]
pub struct basic_block {
    pub instructions: Vec<NodeRef>,
    pub successors: Vec<block_reference>,
    pub predecessors: Vec<block_reference>,
    pub loop_depth: u32,
}

#[derive(Debug)]
pub struct lir_function {
    pub blocks: SlotMap<block_reference, basic_block>,
    pub entry: block_reference,
    pub graph: LirGraph, //value graph
}

//virtual registers that are live at the beginning and enmd of each block
pub struct liveness {
    pub live_in: SecondaryMap<block_reference, HashSet<virtual_register>>,
    pub live_out: SecondaryMap<block_reference, HashSet<virtual_register>>,
}

impl liveness {

    //handles edge case where we have predecessors owning phis instead of
    //the block that contains the phi
    pub fn check_phis(func: &lir_function) -> SecondaryMap<block_reference,
        Vec<(block_reference, virtual_register)>> {
        //map block --> <predecessor, virtual register>
        //we use this to determine which virtual registers we need
        //because of the aforementioned issue
        let mut result: SecondaryMap<block_reference,
            Vec<(block_reference, virtual_register)>> = SecondaryMap::new();

        for (block_ref, block) in &func.blocks {
            for &node_ref in &block.instructions {
                let node = func.graph.node(node_ref);
                if !matches!(node.kind, NodeKind::Phi) {
                    break;
                }

                for (i, input) in node.inputs.iter().enumerate() {
                    if i == 0 { //control edge so we skip
                        continue;
                    }

                    let predecessor_index = i - 1; //input --> predecessor index

                    //record predecessor for virtual reg. needed for phi
                    if let Some(&predecessor) = block.predecessors.get(predecessor_index) {
                        if let Some(vreg) = func.graph.get_virtual_register(input.node) {
                            result.entry(block_ref).unwrap().or_default().push((predecessor, vreg));
                        }
                    }
                }
            }
        }

        return result;

    }

    pub fn liveness(func: &lir_function) -> Self {
        let mut live_in: SecondaryMap<block_reference, HashSet<virtual_register>> = SecondaryMap::new();
        let mut live_out: SecondaryMap<block_reference, HashSet<virtual_register>> = SecondaryMap::new();
        for (block_ref, _) in &func.blocks {
            live_in.insert(block_ref, HashSet::new());
            live_out.insert(block_ref, HashSet::new());
        }

        let phis: SecondaryMap<block_reference,
            Vec<(block_reference, virtual_register)>> = Self::check_phis(func);

        let mut changed = true;
        while changed {
            changed = false;


            for (block_ref, block) in &func.blocks {
                //union live_in of successors
                let mut new_out: HashSet<virtual_register> = block.successors.iter().
                    flat_map(|&s| live_in[s].iter().copied())
                    .collect();

                //adds phis that the block is responsible for
                for &successor in &block.successors {
                    if let Some(phi) = phis.get(successor) {
                        for &(pred, vreg) in phi {
                            if pred == block_ref {
                                new_out.insert(vreg);
                            }
                        }
                    }
                }

                let mut live = new_out.clone();

                for &node_ref in block.instructions.iter().rev() {
                    let node = func.graph.node(node_ref);

                    //if we have a phi, remove its definition (since it can't be live)
                    if matches!(node.kind, NodeKind::Phi) {
                        if let Some(definition) = node.virtual_register {
                            live.remove(&definition);
                        }

                        continue;
                    }

                    //if instruction gives value, we know it can't be live, so remove
                    if let Some(definition) = node.virtual_register {
                        live.remove(&definition);
                    }

                    //else it has to be live, so we add it to the live list
                    for input in &node.inputs {
                        if let Some(vreg) = func.graph.get_virtual_register(input.node) {
                            live.insert(vreg);
                        }
                    }


                }

                let new_in = live;
                if new_out != live_out[block_ref] {
                    live_out[block_ref] = new_out;
                    changed = true;
                }

                if new_in != live_in[block_ref] {
                    live_in[block_ref] = new_in;
                    changed = true;
                }
            }
        }

        return liveness {
            live_in, live_out,
        }
    }
}

pub struct interference_graph {
    adj: HashMap<virtual_register, HashSet<virtual_register>>,

    pub degree: HashMap<virtual_register, usize>,
}

impl interference_graph {
    fn new() -> Self {
        Self {adj: HashMap::new(), degree: HashMap::new()}
    }

    fn add_edge(&mut self, source: virtual_register, destination: virtual_register) {
        if source == destination {
            return;
        }

        if self.adj.get(&source).map_or(false, |adj| adj.contains(&destination)) {
            return;
        }

        self.adj.entry(source).or_default().insert(destination);
        self.adj.entry(destination).or_default().insert(source);
        *self.degree.entry(source).or_default() += 1;
        *self.degree.entry(destination).or_default() += 1;

    }

    pub fn get_neighbors(&self, destination: virtual_register) -> impl Iterator<Item= virtual_register> + '_ {
        self.adj.get(&destination).into_iter().flat_map(|adj| adj.iter().cloned())
    }

    pub fn get_nodes(&self) -> impl Iterator<Item=virtual_register> + '_ {
        self.adj.keys().cloned()
    }

    pub fn make_interference_graph(func: &lir_function, liveness: &liveness) -> Self {
        let mut interference_graph = Self::new();

        for (block_ref, block) in &func.blocks {
            let mut live: HashSet<virtual_register> = liveness.live_out[block_ref].clone();

            for &node_ref in block.instructions.iter().rev() {
                let node = func.graph.node(node_ref);

                if let Some(definition) = node.virtual_register {
                    interference_graph.adj.entry(definition).or_default();
                    interference_graph.degree.entry(definition).or_default();
                }

                if matches!(node.kind, NodeKind::Phi) {
                    if let Some(definition) = node.virtual_register {
                        //definitions interfere with everything live
                        for &live_ in &live {
                            interference_graph.add_edge(definition, live_);
                        }
                        live.remove(&definition);
                    }
                    continue; //don't add phis, since they belong to predecessors

                }


                if let Some(definition) = node.virtual_register {
                    //definitions interfere with everything live
                    for &live_ in &live {
                        interference_graph.add_edge(definition, live_);
                    }
                    live.remove(&definition);
                }

                for input in &node.inputs {
                    if let Some(vreg) = func.graph.get_virtual_register(input.node) {
                        live.insert(vreg);
                    }
                }
            }
        }

        return interference_graph;
    }
}

fn spill_cost(func: &lir_function) -> HashMap<virtual_register, f64> {
    let mut cost: HashMap<virtual_register, f64> = HashMap::new();

    for (_, block) in &func.blocks {
        //weighted cost because we assume deeper loops are exponentially more expensive
        //since instructions inside loops execute more often
        let modifier = 10f64.powi(block.loop_depth as i32);
        for &node_ref in &block.instructions {
            let node = func.graph.node(node_ref);

            //produced value is a cost because we need a store after every definition
            if let Some(definition) = node.virtual_register {
                *cost.entry(definition).or_default() += modifier;

            }

            //virtual register is a cost because we need a load before use
            for input in &node.inputs {
                if let Some(vreg) = func.graph.get_virtual_register(input.node) {
                    *cost.entry(vreg).or_default() += modifier;
                }
            }

        }
    }
    return cost;
}

pub struct register_allocator {
    pub k: usize,
}

impl register_allocator {
    pub fn new (k: usize) -> Self {
        Self {
            k,
        }
    }

    //colors an interference graph
    pub fn color(&self,
                 interference_graph: &interference_graph, cost: &HashMap<virtual_register, f64>)
        -> coloring_result {

        //ordered removal stack
        let virtual_register_stack = self.simplify(interference_graph, cost);

        let mut coloring: HashMap<virtual_register, usize> = HashMap::new();
        let mut spills: Vec<virtual_register> = Vec::new();

        //last removed is colored first because it was in least constrained graph
        for (vreg, is_candidate) in virtual_register_stack.into_iter().rev() {
            let used_colors: HashSet<usize> =
                interference_graph.get_neighbors(vreg).
                    filter_map(|neighbor| coloring.get(&neighbor).copied()).collect();

            //find first color that isn't already used by a neighbor
            match (0..self.k).find(|color| !used_colors.contains(color)) {
                //trivial case
                Some(color_) => {
                    coloring.insert(vreg, color_);
                },
                //optimistic spilling, so we add to spills and try again later
                None => {
                    spills.push(vreg);
                }
            }
        }

        if spills.is_empty() {
            return coloring_result::success(coloring);
        } else {
            return coloring_result::failure(spills);
        }




    }

    //makes ordered removal stack; we remove low-degree nodes first
    //otherwise, we optimistically choose a candidate for spilling
    fn simplify(&self, interference_graph: &interference_graph,
                cost: &HashMap<virtual_register, f64>) -> Vec<(virtual_register, bool)> {
        let mut degree: HashMap<virtual_register, usize> = interference_graph.degree.clone();
        let mut remaining_nodes: HashSet<virtual_register> =
            interference_graph.get_nodes().collect();
        let mut remove_stack: Vec<(virtual_register, bool)> = Vec::new();

        while !remaining_nodes.is_empty() {
            //find nodes with degree < k
            //these are guaranteed to get a color so we don't need to spill
            let remove = remaining_nodes.iter().copied().find(|vreg| degree[vreg] < self.k);
            if let Some(vreg) = remove {
                Self::remove(vreg, &mut remaining_nodes, &mut degree, interference_graph);
                remove_stack.push((vreg, false));
            } else {

                //remaining nodes have degree >= k, so we pick lowest cost/degree as spill
                //we still push it on the spill stack anyway because it could still get a color
                let spill_candidate = remaining_nodes.iter().copied().min_by(|&a, &b| {
                    let candidate_a = cost.get(&a).copied().unwrap();
                    let candidate_b = cost.get(&b).copied().unwrap();
                    let degree_a = degree[&a] as f64;
                    let degree_b = degree[&b] as f64;
                    //cost divided by degree, we just cross-multiply to avoid floats
                    (candidate_a * degree_b).partial_cmp(&(candidate_b * degree_a)).unwrap()
                }).unwrap();

                Self::remove(spill_candidate, &mut remaining_nodes,
                             &mut degree, interference_graph);
                remove_stack.push((spill_candidate, true));

            }

        }
        return remove_stack;
    }

    //removes node from interference graph and updates degrees
    fn remove(vreg: virtual_register,
                               remaining_nodes: &mut HashSet<virtual_register>,
                               degree: &mut HashMap<virtual_register, usize>,
                               interference_graph: &interference_graph) {
        remaining_nodes.remove(&vreg);
        for neighbor in interference_graph.get_neighbors(vreg) {
            if remaining_nodes.contains(&neighbor) {
                degree.entry(neighbor).
                    and_modify(|neighbor_deg| *neighbor_deg = *neighbor_deg - 1);
            }
        }

    }
}

fn spill(func: &mut lir_function, spills: &Vec<virtual_register>) {
    let mut spill_slots: HashMap<virtual_register, spill_slot> = HashMap::new();
    for &virtual_register in spills {
        let slot = func.graph.new_spill_slot();
        spill_slots.insert(virtual_register, slot);
    }

    for (_, block) in &mut func.blocks {
        let old_instructions = std::mem::take(&mut block.instructions);
        let mut new_instructions: Vec<NodeRef> = Vec::new();

        for node_ref in old_instructions {
            let inputs: Vec<_> = func.graph.node(node_ref).inputs.iter()
                .map(|input| (input.node, func.graph.get_virtual_register(input.node)))
                .collect();

            for (_, vreg) in &inputs {
                if let Some(v) = vreg {
                    if let Some(&slot) = spill_slots.get(v) {
                        let new_vreg = func.graph.new_virtual_register();
                        let load = func.graph.add_node(NodeKind::load_spill(slot));
                        func.graph.set_virtual_register(load, new_vreg);
                        new_instructions.push(load);
                    }
                }

            }

            new_instructions.push(node_ref);

            if let Some(vreg) = func.graph.node(node_ref).virtual_register {
                if let Some(&slot) = spill_slots.get(&vreg) {
                    let store = func.graph.add_node(NodeKind::store_spill(slot));
                    new_instructions.push(store);
                }
            }


        }

        block.instructions = new_instructions;

    }
}


pub fn chaitin_briggs(func: &mut lir_function, register_count: usize) -> HashMap<virtual_register, usize> {
    let register_allocator = register_allocator::new(register_count);

    //iterate until we find a successful coloring
    loop {
        let liveness = liveness::liveness(&func);
        let interference_graph = interference_graph::make_interference_graph(func, &liveness);
        let spill_cost = spill_cost(func);

        match register_allocator.color(&interference_graph, &spill_cost) {
            coloring_result::success(coloring) => {
                return coloring;
            }
            coloring_result::failure(spills) => {
                spill(func, &spills);
            }
        }
    }
}
