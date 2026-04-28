use std::collections::HashMap;
use std::fmt::Write;

use crate::ir::{
    FunctionHandle, Module,
    lir::code::{LirGraph, NodeKind, NodeRef, NodeRefLike},
    pass::{FunctionPass, lir_body},
};

/*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PortRole {
    Control,
    Memory,
    Phantom,
    Data,
}

#[derive(Default)]
struct DefaultDotMetadata;

trait DotMetadata {
    fn node_title(&self, graph: &LirGraph, node: NodeRef) -> String;
    fn input_port(&self, graph: &LirGraph, node: NodeRef, idx: usize) -> (String, PortRole);
    fn output_port(&self, graph: &LirGraph, node: NodeRef, idx: usize) -> (String, PortRole);
}

impl DotMetadata for DefaultDotMetadata {
    fn node_title(&self, graph: &LirGraph, node: NodeRef) -> String {
        match node.kind() {
            NodeKind::Start => "op: start".to_string(),
            NodeKind::Stop => "op: stop".to_string(),
            NodeKind::Call { arity } => format!("op: call (arity={arity})"),
            NodeKind::Add => "op: add".to_string(),
            NodeKind::Sub => "op: sub".to_string(),
            NodeKind::SMul => "op: smul".to_string(),
            NodeKind::UMul => "op: umul".to_string(),
            NodeKind::SDivMod => "op: sdivmod".to_string(),
            NodeKind::UDivMod => "op: udivmod".to_string(),
            NodeKind::Phi => "op: phi".to_string(),
            NodeKind::Ret => "op: ret".to_string(),
            NodeKind::If => "op: if".to_string(),
            NodeKind::CProj => "op: cproj".to_string(),
            NodeKind::Merge => "op: merge".to_string(),
            NodeKind::Nil => "op: nil".to_string(),
        }
    }

    fn input_port(&self, graph: &LirGraph, node: NodeRef, idx: usize) -> (String, PortRole) {
        match graph.node_kind(node) {
            NodeKind::If => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                1 => ("pred".to_string(), PortRole::Data),
                _ => (format!("in{idx}"), PortRole::Data),
            },
            NodeKind::CProj => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                _ => (format!("in{idx}"), PortRole::Data),
            },
            NodeKind::Merge => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                _ => (format!("in{idx}"), PortRole::Data),
            },
            NodeKind::Ret => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                1 => ("val".to_string(), PortRole::Data),
                _ => (format!("in{idx}"), PortRole::Data),
            },
            NodeKind::Call { .. } => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                _ => (format!("arg{}", idx - 1), PortRole::Data),
            },
            _ => (format!("in{idx}"), PortRole::Data),
        }
    }

    fn output_port(&self, graph: &LirGraph, node: NodeRef, idx: usize) -> (String, PortRole) {
        match graph.node_kind(node) {
            NodeKind::Start => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                1 => ("mem".to_string(), PortRole::Memory),
                _ => (format!("arg{}", idx - 2), PortRole::Data),
            },
            NodeKind::If => match idx {
                0 => ("true".to_string(), PortRole::Control),
                1 => ("false".to_string(), PortRole::Control),
                _ => (format!("out{idx}"), PortRole::Data),
            },
            NodeKind::CProj => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                _ => (format!("out{idx}"), PortRole::Data),
            },
            NodeKind::Merge => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                _ => (format!("out{idx}"), PortRole::Data),
            },
            NodeKind::Ret => match idx {
                0 => ("ctrl".to_string(), PortRole::Control),
                1 => ("mem".to_string(), PortRole::Memory),
                _ => (format!("out{idx}"), PortRole::Data),
            },
            NodeKind::SDivMod | NodeKind::UDivMod => match idx {
                0 => ("quo".to_string(), PortRole::Data),
                1 => ("rem".to_string(), PortRole::Data),
                _ => (format!("out{idx}"), PortRole::Data),
            },
            NodeKind::Nil => match idx {
                0 => ("nil".to_string(), PortRole::Data),
                _ => (format!("out{idx}"), PortRole::Data),
            },
            _ => {
                if idx == 0 {
                    ("res".to_string(), PortRole::Data)
                } else {
                    (format!("res{idx}"), PortRole::Data)
                }
            }
        }
    }
}

fn escape_dot_record_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' | '"' | '{' | '}' | '<' | '>' | '|' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn record_cell(port: Option<&str>, text: impl AsRef<str>) -> String {
    let text = escape_dot_record_text(text.as_ref());
    match port {
        Some(port) => format!("<{port}>{text}"),
        None => text,
    }
}

fn edge_attrs(src: PortRole, dst: PortRole) -> &'static str {
    use PortRole::*;

    match (src, dst) {
        (Control, _) | (_, Control) => "[color=red]",
        (Memory, _) | (_, Memory) => "[style=dashed]",
        (Phantom, _) | (_, Phantom) => "[style=dotted]",
        _ => "",
    }
}

fn node_label<M: DotMetadata>(graph: &LirGraph, node: NodeRef, meta: &M) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();

    let input_count = graph.input_count(node);
    if input_count != 0 {
        let mut row = Vec::with_capacity(input_count);
        for idx in 0..input_count {
            let (name, _) = meta.input_port(graph, node, idx);
            row.push(record_cell(Some(&format!("in{idx}")), name));
        }
        rows.push(row);
    }

    rows.push(vec![record_cell(None, meta.node_title(graph, node))]);

    let output_count = graph.output_count(node);
    if output_count != 0 {
        let mut row = Vec::with_capacity(output_count);
        for idx in 0..output_count {
            let (name, _) = meta.output_port(graph, node, idx);
            let ty = format!("{:?}", graph.output_type(node, idx));
            row.push(record_cell(
                Some(&format!("out{idx}")),
                format!("{name}: {ty}"),
            ));
        }
        rows.push(row);
    }

    let rendered_rows = rows
        .into_iter()
        .filter(|row| !row.is_empty())
        .map(|row| format!("{{ {} }}", row.join(" | ")))
        .collect::<Vec<_>>()
        .join(" | ");

    format!("{{ {rendered_rows} }}")
}

pub struct DumpGraph<W: Write> {
    out: W,
}

impl<W: Write> DumpGraph<W> {
    pub fn new(out: W) -> Self {
        Self { out }
    }
}

impl<W: Write> FunctionPass for DumpGraph<W> {
    fn apply_function(&mut self, module: &mut Module, func: FunctionHandle) {
        let graph = lir_body(module, func);
        let meta = DefaultDotMetadata::default();

        let _ = writeln!(self.out, "digraph lir_function {{");
        let _ = writeln!(self.out, "\trankdir=\"BT\"");

        let mut ids: HashMap<NodeRef, usize> = HashMap::new();
        for (idx, (node, _)) in graph.nodes.iter().enumerate() {
            ids.insert(node, idx);
        }

        for (node, _) in graph.nodes.iter() {
            let id = ids[&node];
            let label = node_label(graph, node, &meta);
            let _ = writeln!(
                self.out,
                "\tn{id} [shape=record,shape=Mrecord,label=\"{label}\"];"
            );
        }

        for (dst, _) in graph.nodes.iter() {
            let dst_id = ids[&dst];
            let dst_input_count = dst.inputs(graph).count();

            for in_idx in 0..dst_input_count {
                let Some(src_use) = graph.input_use(dst, in_idx) else {
                    continue;
                };

                if src_use.out_idx >= graph.output_count(src_use.node) {
                    continue;
                }

                let src_id = ids[&src_use.node];

                let (_, src_role) = meta.output_port(graph, src_use.node, src_use.out_idx);
                let (_, dst_role) = meta.input_port(graph, dst, in_idx);
                let attrs = edge_attrs(src_role, dst_role);

                if attrs.is_empty() {
                    let _ = writeln!(
                        self.out,
                        "\tn{src_id}:out{} -> n{dst_id}:in{};",
                        src_use.out_idx, in_idx
                    );
                } else {
                    let _ = writeln!(
                        self.out,
                        "\tn{src_id}:out{} -> n{dst_id}:in{} {};",
                        src_use.out_idx, in_idx, attrs
                    );
                }
            }
        }

        let _ = writeln!(self.out, "}}");
    }
}*/
