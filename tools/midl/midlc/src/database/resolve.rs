use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::ast;

use super::Context;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Version;

#[derive(Debug, Clone, Default)]
// Per-node information for the version graph.
struct NodeInfo {
    // Set of points at which to split this element in the final decomposition.
    // It initially contains 2 endpoints (or 3 points with deprecation), and
    // then receives more points from incoming neighbours.
    points: BTreeSet<Version>,
    // Set of outgoing neighbors. These are either *membership edges* (from
    // child to parent, e.g. struct member to struct) or *reference edges* (from
    // declaration to use, e.g. struct to table member carrying the struct).
    neighbors: BTreeSet<&'static ast::Element>,
}

pub struct ResolveStep {
    graph: BTreeMap<&'static ast::Element, NodeInfo>,
}

impl ResolveStep {
    pub fn new() -> Self {
        Self { graph: BTreeMap::new() }
    }

    // This step resolves all references in the library. It does so in three steps:
    //
    // 1. Parse the structure of each reference. For example, given `foo.bar`, this
    //    means choosing between "library foo, decl bar" and "decl foo, member bar".
    //    This step does not consult availabilities nor the version selection.
    // 2. Perform temporal decomposition, splitting declarations into finer-grained
    //    pieces such that for each one, nothing changes over its availability.
    // 3. Resolve all references in the decomposed AST, linking each one to the
    //    specific Element* it refers to.
    //
    // Note that ResolveStep does not resolve constant values (i.e. calling
    // Constant::ResolveTo). That happens in the CompileStep.
    pub(crate) fn run(&self, ctx: Rc<RefCell<Context<'_>>>) {
        let ctx = ctx.borrow_mut();

        /*{
            // In a single pass:
            // (1) parse all references into keys/contextuals;
            // (2) insert reference edges into the graph.
            ctx.library.traverse_elements(|element: &Element| {
                visit_element(element, Context(Context::Mode::kParseAndInsert, element));
            });

            // Add all elements of this library to the graph, with membership edges.
            for entry in ctx.library.declarations.all {
                let decl = entry.second;
                // Note: It's important to insert decl here so that (1) we properly
                // initialize its points in the next loop, and (2) we can always recursively
                // look up a neighbor in the graph, even if it has out-degree zero.
                let idx = self.graph.insert(decl, NodeInfo::default());

                decl.for_each_member(|member: &Element| {
                    self.graph[member].neighbors.insert(member);
                });
            }

            // Initialize point sets for each element in the graph.
            for (element, info) in self.graph.iter() {
                // There shouldn't be any library elements in the graph because they are
                // special (they don't get split, so their availabilities stop at
                // kInherited). We don't add membership edges to them, and we specifically
                // avoid adding reference edges to them in ResolveStep::ParseReference.
                assert!(element.kind != Element::Kind::kLibrary);

                // Each element starts with between 2 and 5 points. All have (1) `added` and
                // (2) `removed`. Some have (3) `deprecated`. Some are added back for legacy
                // support, so they have (4) LEGACY and (5) +inf. Elements from other
                // libraries (that exist due to reference edges) only ever have 2 points
                // because those libraries are already compiled, hence post-decomposition.
                info.points = element.availability.points();
            }

            // Run the temporal decomposition algorithm.
            let worklist: Vec<&Element> = vec![];
            worklist.reserve(self.graph.len());
            for (element, info) in self.graph.iter() {
                worklist.push_back(element);
            }

            while worklist.len() != 0 {
                let element = worklist.last().unwrap();
                worklist.pop();
                let NodeInfo { points, neighbors } = self.graph[element];

                for neighbor in neighbors {
                    let neighbor_points = self.graph[neighbor].points;
                    let min = neighbor_points.first().unwrap();
                    let max = neighbor_points.last().unwrap();
                    let pushed_neighbor = false;

                    for p in points {
                        if p > min && p < max {
                            let inserted = neighbor_points.insert(p);
                            if inserted && !pushed_neighbor {
                                worklist.push(neighbor);
                                pushed_neighbor = true;
                            }
                        }
                    }
                }
            }
        }*/

        // Resolve all references and validate them.
        ctx.library.traverse_elements(&mut |element| {
            self.visit_element(element);
            //    self.visit_element(element, Context(Context::Mode::kResolveAndValidate, element));
        });
    }

    fn visit_element(&self, element: &ast::Element) {
        println!("visit_element: {:?}", element);
    }
}
