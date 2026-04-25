//python toy example but ported to rust yayyy

//implementation based on the paper "A Simple, Fast Dominance Algorithm"
//by Cooper et al. (2001)
use slotmap::SecondaryMap;
use crate::ir::hir::code::{Label, HirFunctionBody};

pub struct dominator_tree {
    pub immediate_dominator: SecondaryMap<Label, Label>,
}

impl dominator_tree {
    pub fn make_dominator_tree(body: &HirFunctionBody) -> Self{
        //label to usize index mapping
        let labels: Vec<Label> = body.blocks().map(|(l, _) | l).collect();
        let mut label_to_index = SecondaryMap::<Label, usize>::new();
        for (i, &l) in labels.iter().enumerate() {
            label_to_index.insert(l, i);
        }

        let node_count = labels.len();
        let entry_index = label_to_index[body.entry()];
        let mut successors = vec![vec![]; node_count];
        let mut predecessors = vec![vec![]; node_count];

        for (source, block) in body.blocks() {
            for destination in block.successors() {
                let source_ = label_to_index[source];
                let dest_ = label_to_index[destination];
                successors[source_].push(dest_);
                predecessors[dest_].push(source_);
            }

        }

        //rpo + computes dominators to build dominator tree
        let (rpo, rpo_index, _) = reverse_postorder(&successors, entry_index, node_count);
        let immediate_dominator_index = dominator_search(&predecessors, entry_index, &rpo,
                                                         &rpo_index, node_count);

        let mut immediate_dominator = SecondaryMap::<Label, Label>::new();
        for (node_index, immediate_dominator_) in immediate_dominator_index.iter().enumerate() {
            if let Some(immediate_dominator_) = immediate_dominator_ {
                immediate_dominator.insert(labels[node_index], labels[*immediate_dominator_]);
            }
        }

        return (dominator_tree {immediate_dominator})
    }

    //walks up dominator tree from node following idoms, returns whether it finds dom
    pub fn dominates(&self, dominator: Label, node: Label) -> bool {
        let mut current = node;
        loop {
            if current == dominator {
                return true;
            }

            match self.immediate_dominator.get(current) {
                Some(parent) => current = *parent,
                None => return false
            }
        }
    }

    //finds idom of node
    //entry dominates itself so we return none
    pub fn immediate_dominator(&self, label: Label) -> Option<Label> {
        let immediate_dominator_ = match self.immediate_dominator.get(label).copied() {
            Some(immediate_dominator_) => immediate_dominator_,
            None => return None,
        };

        if immediate_dominator_ == label {
            None
        } else {
            Some(immediate_dominator_)
        }
    }
}


//dfs --> reverse to determine rpo order
//gives traversal order + node indices in rpo + loop headers
//we don't technically need loop headers, this is from the failed optimization
fn reverse_postorder(successors: &[Vec<usize>],
                     entry: usize, node_count: usize) -> (Vec<usize>, Vec<usize>, Vec<bool>) {
    //let node_count = cfg.node_count;
    let mut rpo = Vec::new();
    let mut rpo_index = vec![0usize; node_count];
    let mut stack = vec![false; node_count];
    let mut done = vec![false; node_count];
    let mut loop_headers = vec![false; node_count];

    //recursive dfs, adds nodes to rpo
    fn depth_first_search(successors: &[Vec<usize>], current: usize,
                          stack: &mut [bool], done: &mut [bool],
                          loop_headers: &mut [bool], rpo: &mut Vec<usize>,) {
        stack[current] = true;

        for &next in &successors[current] {
            if !done[next] { //skip if done
                if stack[next] { //we have back edge, so mark as loop header
                    //this is unnecessary, can probably remove
                    loop_headers[next] = true;
                } else { //unvisited, recurse
                    depth_first_search(successors, next, stack, done, loop_headers, rpo);
                }
            }
        }

        stack[current] = false;
        done[current] = true;
        rpo.push(current);

    }

    depth_first_search(successors, entry, &mut stack, &mut done, &mut loop_headers, &mut rpo);
    rpo.reverse();

    for (i, &node) in rpo.iter().enumerate() {
        rpo_index[node] = i; //indexing
    }

    return (rpo, rpo_index, loop_headers)

}

//we iterate over nodes in rpo repeatedly until convergence (nothing changes)
//for each node, we compute idom by intersect on doms of predecessors
fn dominator_search(predecessors: &[Vec<usize>], entry: usize, rpo: &[usize],
                    rpo_index: &[usize], node_count: usize) -> Vec<Option<usize>> {
    //let (rpo, rpo_index, _) = reverse_postorder(cfg, entry);

    let mut immediate_dominator = vec![None; node_count];
    immediate_dominator[entry] = Some(entry);

    //finds common dom of two nodes by walking up dom tree
    fn intersect(mut a: usize, mut b: usize, immediate_dominator: &[Option<usize>],
                 rpo_index: &[usize]) -> usize {
        while a != b { //not met so we walk deepest node up
            while rpo_index[a] > rpo_index[b] {
                a = immediate_dominator[a].unwrap();
            }
            while rpo_index[b] > rpo_index[a] {
                b = immediate_dominator[b].unwrap();
            }
        }
        return (a)
    }

    //finds idom of a single node with intersect on predecessors
    fn new_immediate_dominator(predecessors: &[Vec<usize>], node: usize,
                               immediate_dominator: &[Option<usize>],
                               rpo_index: &[usize],) -> Option<usize> {
        let mut predecessors = predecessors[node].iter().copied().filter(|&p| immediate_dominator[p].is_some());

        let mut result = match predecessors.next() {
            Some(x) => x,
            None => return None,
        };

        for p in predecessors {
            result = intersect(p, result, immediate_dominator, rpo_index);
        }

        Some(result)


    }

    loop {
        let mut changed = false;

        for &node in &rpo {
            if node == entry { //skip entry node
                continue;
            }

            //gives idom of node in current state (update basically)
            let new_idom = new_immediate_dominator(predecessors, node, &immediate_dominator, &rpo_index);
            if immediate_dominator[node] != new_idom {
                immediate_dominator[node] = new_idom;
                changed = true;
            }

        }

        if !changed { //converge
            break;
        }

    }
    return (immediate_dominator)


}
// struct CFG {
//     successors: Vec<Vec<usize>>,
//     predecessors: Vec<Vec<usize>>,
//     node_count: usize,
// }
//
// impl CFG {
//     fn new(node_count: usize) -> Self {
//         CFG {
//             successors: vec![vec![]; node_count],
//             predecessors: vec![vec![]; node_count],
//             node_count,
//
//         }
//     }
//
//     fn add_edge(&mut self, source: usize, destination: usize) {
//         self.successors[source].push(destination);
//         self.predecessors[destination].push(source);
//     }
// }

// fn make_nested(depth: usize) -> (CFG, usize) {
//     let node_count = depth + 3;
//     let entry = 0;
//     let body = depth + 1;
//     let exit = depth + 2;
//     let node = |i: usize| i + 1;
//     let mut cfg = CFG::new(node_count);
//
//     cfg.add_edge(entry, node(0));
//     for i in 0..(depth - 1) {
//         cfg.add_edge(node(i), node(i + 1));
//     }
//
//     cfg.add_edge(node(depth - 1), body);
//     cfg.add_edge(body, node(depth - 1));
//
//     for i in (1..depth).rev() {
//         cfg.add_edge(node(i), node(i - 1));
//     }
//
//     cfg.add_edge(node(0), exit);
//     (cfg, entry)
// }