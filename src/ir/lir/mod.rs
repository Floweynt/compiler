use smallvec::SmallVec;

mod types;

#[repr(transparent)]
struct NodeRef(usize);

struct NodeUse {
    node: NodeRef,
    out_idx: usize,
}

struct NodeDef {
    node: NodeRef,
    in_idx: usize,
}

struct Node {
    inputs: SmallVec<[NodeUse; 4]>,
    outputs: SmallVec<[SmallVec<[NodeDef; 4]>; 2]>,
}

struct Function {
    nodes: Vec<Node>,
}
