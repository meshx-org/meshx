use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use super::Context;
use crate::{
    ast::{self, Declaration},
    diagnotics::DiagnosticsError,
};

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
    neighbors: BTreeSet<ast::Element>,
}

enum ResolveMode {
    // Calls ParseReference and InsertReferenceEdges.
    ParseAndInsert,
    // Calls ResolveReference and ValidateReference.
    ResolveAndValidate,
}

struct ResolveContext {
    // What to do when we reach leaves (references).
    mode: ResolveMode,

    /// Used in kParseAndInsert. If true, we call Reference::MarkContextual
    /// instead of Reference::MarkFailed for a single-component reference,
    /// deferring the final contextual lookup to kResolve.
    allow_contextual: bool,

    /// Element that the reference occurs in.
    enclosing: ast::Element,
}

struct Lookup<'a, 'ctx, 'd> {
    step: &'a ResolveStep<'ctx, 'd>,
    reference: &'a ast::Reference,
}

impl<'a, 'ctx, 'd> Lookup<'a, 'ctx, 'd> {
    fn new(step: &'a ResolveStep<'ctx, 'd>, reference: &'a ast::Reference) -> Self {
        Self { step, reference }
    }

    fn try_library(&self, name: &Vec<String>) -> Option<Rc<ast::Library>> {
        let deps = self.step.ctx.library.dependencies.borrow();
        let root_lib = self.step.ctx.all_libraries.borrow().root_library();

        if let Some(lib_name) = root_lib.name.get() {
            if name == lib_name {
                return Some(root_lib);
            }
        }

        let source = self.reference.span.clone().unwrap().source;
        deps.lookup_and_mark_used(source, name)
    }

    fn try_decl(&self, library: &Rc<ast::Library>, name: &String) -> Option<ast::ReferenceKey> {
        let all_decl = &library.declarations.borrow().all;
        let results = all_decl.get_vec(name);

        if results.is_none() {
            return None;
        }

        let results = results.unwrap();

        // try_decl is only used from within parse_sourced_reference, which should not
        // resolve Internal declarations names; only synthetic references can
        // resolve internal names.
        // Internal declarations should only exist in the root library, and should
        // never have conflicting names, so any match should have only one element.
        // We therefore return None if any of the declarations found is an
        // internal one.

        for decl in results.iter() {
            if let ast::Declaration::Builtin { decl } = decl {
                if decl.borrow().is_internal() {
                    return None;
                }
            }
        }

        return Some(ast::ReferenceKey {
            library: library.clone(),
            decl_name: name.to_string(),
            member_name: None,
        });
    }

    fn must_decl(&self, library: &Rc<ast::Library>, name: &String) -> Option<ast::ReferenceKey> {
        if let Some(key) = self.try_decl(library, name) {
            return Some(key);
        }

        // TODO: use diagnostics instead
        panic!("ErrNameNotFound {:?} {} {:?}", self.reference.span, name, library.name);
        None
    }

    fn try_member(&self, parent: ast::Declaration, name: &String) -> Option<ast::Element> {
        match parent {
            // TODO: bits and enums
            _ => None,
        }
    }

    fn must_member(&self, parent: ast::Declaration, name: &String) -> Option<ast::Element> {
        match parent {
            ast::Declaration::Bits | ast::Declaration::Enum => {
                if let Some(member) = self.try_member(parent, name) {
                    return Some(member);
                }
            }

            _ => {
                panic!("ErrCannotReferToMember");
                //Fail(ErrCannotReferToMember, ref_.span(), parent);
                return None;
            }
        }
        panic!("ErrMemberNotFound");
        //  Fail(ErrMemberNotFound, ref_.span(), parent, name);
        return None;
    }
}

/// This step resolves all references in the library. It does so in three steps:
///
/// 1. Parse the structure of each reference. For example, given `foo.bar`, this
///    means choosing between "library foo, decl bar" and "decl foo, member bar".
///    This step does not consult availabilities nor the version selection.
/// 2. Perform temporal decomposition, splitting declarations into finer-grained
///    pieces such that for each one, nothing changes over its availability.
/// 3. Resolve all references in the decomposed AST, linking each one to the
///    specific Element* it refers to.
///
/// Note that ResolveStep does not resolve constant values (i.e. calling
/// Constant::ResolveTo). That happens in the CompileStep.
pub(crate) struct ResolveStep<'ctx, 'd> {
    ctx: &'ctx mut Context<'d>,

    /// The version graph for this library: directed, possibly cyclic, possibly
    /// disconnected. Contains only elements from the current library's platform:
    /// all the current library's elements, plus elements from external libraries
    /// it references. The latter have in-degree zero, i.e. they only appear as map
    /// keys and never in the sets of outgoing neighbors.
    graph: RefCell<BTreeMap<ast::Element, NodeInfo>>,
}

impl<'ctx, 'd> ResolveStep<'ctx, 'd> {
    pub fn new(ctx: &'ctx mut Context<'d>) -> Self {
        Self {
            ctx,
            graph: RefCell::from(BTreeMap::new()),
        }
    }

    pub(crate) fn run(&self) -> bool {
        let checkpoint = self.ctx.diagnostics.checkpoint();
        self.run_impl();
        checkpoint.no_new_errors()
    }

    fn run_impl(&self) {
        // In a single pass:
        // (1) parse all references into keys/contextuals;
        // (2) insert reference edges into the graph.
        self.ctx.library.traverse_elements(&mut |element| {
            self.visit_element(
                element.clone(),
                &ResolveContext {
                    mode: ResolveMode::ParseAndInsert,
                    enclosing: element,
                    allow_contextual: false,
                },
            );
        });

        {
            let mut graph = self.graph.borrow_mut();
            println!("d {:?}", graph);

            // Add all elements of this library to the graph, with membership edges.
            for (_, decl) in self.ctx.library.declarations.borrow().all.flat_iter() {
                let el = decl.clone().into();
                // Note: It's important to insert decl here so that (1) we properly
                // initialize its points in the next loop, and (2) we can always recursively
                // look up a neighbor in the graph, even if it has out-degree zero.
                let _idx = graph.insert(el, NodeInfo::default());

                decl.for_each_member(&mut |member: ast::Element| {
                    graph.get_mut(&member).unwrap().neighbors.insert(member);
                });
            }

            // Initialize point sets for each element in the graph.
            /*for (element, info) in graph.iter() {
                // There shouldn't be any library elements in the graph because they are
                // special (they don't get split, so their availabilities stop at
                // kInherited). We don't add membership edges to them, and we specifically
                // avoid adding reference edges to them in ResolveStep::ParseReference.
                assert!(element.kind != ElementKind::Library);

                // Each element starts with between 2 and 5 points. All have (1) `added` and
                // (2) `removed`. Some have (3) `deprecated`. Some are added back for legacy
                // support, so they have (4) LEGACY and (5) +inf. Elements from other
                // libraries (that exist due to reference edges) only ever have 2 points
                // because those libraries are already compiled, hence post-decomposition.
                info.points = element.availability.points();
            } */

            // Run the temporal decomposition algorithm.
            /*let worklist: Vec<&Element> = vec![];
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
            }*/
        }

        // Resolve all references and validate them.
        self.ctx.library.traverse_elements(&mut |element| {
            let context = ResolveContext {
                mode: ResolveMode::ResolveAndValidate,
                allow_contextual: false,
                enclosing: element.clone(),
            };

            self.visit_element(element.clone(), &context);
        });
    }

    fn visit_element(&self, element: ast::Element, context: &ResolveContext) {
        // log::debug!("visit_element: {:?}", element);

        // TODO: attribute parsing
        // for  attribute in element.attributes.attributes {
        //    for (let arg : attribute.args) {
        //        visit_constant(arg.value.get(), context);
        //    }
        // }

        match element {
            ast::Element::Const { inner } => {
                self.visit_type_constructor(&inner.borrow().type_ctor, context);
                self.visit_constant(&inner.borrow().value, context);
            }
            ast::Element::StructMember { inner } => {
                self.visit_type_constructor(&inner.member_type_ctor, context);

                //if (let constant = struct_member.maybe_default_value) {
                //    self.visit_constant(constant.get(), context);
                //}
            }
            ast::Element::ProtocolMethod { inner } => {
                if let Some(type_ctor) = &inner.maybe_request {
                    self.visit_type_constructor(&type_ctor, context);
                }

                if let Some(type_ctor) = &inner.maybe_response {
                    self.visit_type_constructor(&type_ctor, context);
                }
            }
            ast::Element::Alias { inner } => {
                self.visit_type_constructor(&inner.borrow().partial_type_ctor, context);
            }
            ast::Element::Protocol { .. } => {}
            ast::Element::Struct { .. } => {}
            ast::Element::Builtin { .. } => {}
            ast::Element::Bits => {}
            ast::Element::Enum => {}
            ast::Element::Resource => {}
            ast::Element::Protocol { .. } => {}
            ast::Element::Builtin { .. } => {}
            ast::Element::Struct { .. } => {}
            ast::Element::Const { .. } => {}
            ast::Element::NewType => {}
            ast::Element::Table => {}
            ast::Element::Union => {}
            ast::Element::Overlay => {}
            ast::Element::TableMember => {}
            ast::Element::UnionMember => {}
            ast::Element::EnumMember => {}
        }
    }

    fn visit_type_constructor(&self, type_ctor: &ast::TypeConstructor, context: &ResolveContext) {
        self.visit_reference(&type_ctor.layout, context);

        for param in type_ctor.parameters.items.iter() {
            match param {
                ast::LayoutParameter::Literal(_) => {}
                ast::LayoutParameter::Type(type_param) => {
                    self.visit_type_constructor(&type_param.type_ctor, context);
                }
                ast::LayoutParameter::Identifier(identifier_param) => {
                    self.visit_reference(&identifier_param.reference, context);
                    // After resolving an IdentifierLayoutParameter, we can determine
                    // whether it's a type constructor or a constant.
                    if let ast::ReferenceState::Resolved(_) = *identifier_param.reference.state.borrow() {
                        identifier_param.disambiguate();
                    }
                }
            }
        }
    }

    fn visit_constant(&self, constant: &ast::Constant, context: &ResolveContext) {
        match constant {
            ast::Constant::Identifier(identifier_constant) => {
                self.visit_reference(&identifier_constant.reference, context);
            }
            _ => return,
        }
    }

    fn visit_reference(&self, reference: &ast::Reference, context: &ResolveContext) {
        match context.mode {
            ResolveMode::ParseAndInsert => {
                self.parse_reference(reference, context);
                self.insert_reference_edges(reference, context);
            }
            ResolveMode::ResolveAndValidate => {
                self.resolve_reference(reference, context);
                self.validate_reference(reference, context);
            }
        }
    }

    /// Calls ref.set_key, ref.mark_contextual, or ref.mark_failed.
    fn parse_reference(&self, reference: &ast::Reference, context: &ResolveContext) {
        let initial_state = reference.state.clone();
        // let checkpoint = reporter()->Checkpoint();

        match *initial_state.borrow() {
            ast::ReferenceState::RawSynthetic(_) => {
                self.parse_synthetic_reference(reference, context);
            }
            ast::ReferenceState::RawSourced(_) => {
                self.parse_sourced_reference(reference, context);
            }
            ast::ReferenceState::Resolved(_) => {
                // This can only happen for the early compilation of HEAD in
                // AttributeArgSchema::TryResolveAsHead.
                assert!(self.is_resolved_to_head(reference));
                return;
            }
            _ => panic!("unexpected reference state"),
        }

        if reference.state == initial_state {
            // ZX_ASSERT_MSG(checkpoint.NumNewErrors() > 0, "should have reported an error");
            reference.mark_failed();
            return;
        }

        // A library element can reference things via attribute arguments. Other than
        // HEAD, which causes an early return above, we don't allow this: how would we
        // decide what value to use if the const it's referencing has different values
        // at different versions?
        //if context.enclosing.kind == Element::Kind::Library {
        //    Fail(ErrReferenceInLibraryAttribute, reference.span());
        //    reference.mark_failed();
        //    return;
        //}
    }

    fn parse_synthetic_reference(&self, reference: &ast::Reference, context: &ResolveContext) {
        let name = reference
            .raw_synthetic()
            .unwrap()
            .target
            .element()
            .as_decl()
            .expect("only declarations are expected")
            .name();

        reference.set_key(ast::ReferenceKey {
            library: name.library.clone(),
            decl_name: name.decl_name(),
            member_name: None,
        });
    }

    fn parse_sourced_reference(&self, reference: &ast::Reference, context: &ResolveContext) {
        // TOOD(fxbug.dev/77561): Move this information to the FIDL language spec.
        //
        // Below is an outline of FIDL scoping semantics. We navigate it by moving
        // down and to the right. That is, if a rule succeeds, we proceed to the
        // indented one below it (if there is none, SUCCESS); if a rule fails, we try
        // the same-indentation one below it (if there is none, FAIL).
        //
        // - X
        //     - Resolve X as a decl within the current library.
        //     - Resolve X as a decl within the root library.
        //     - Resolve X as a contextual bits/enum member, if context exists.
        // - X.Y
        //     - Resolve X as a decl within the current library.
        //         - Resolve Y as a member of X.
        //     - Resolve X as a library name or alias.
        //         - Resolve Y as a decl within X.
        // - x.Y.Z where x represents 1+ components
        //     - Resolve x.Y as a library name or alias.
        //         - Resolve Z as a decl within x.Y.
        //     - Resolve x as a library name or alias.
        //         - Resolve Y as a decl within x.
        //             - Resolve Z as a member of Y.
        //
        // Note that if you import libraries A and A.B, you cannot refer to bits/enum
        // member [library A].B.C because it is interpreted as decl [library A.B].C.
        // To do so, you must remove or alias one of the imports. We implement this
        // behavior even if [library A.B].C does not exist, since otherwise
        // introducing it would result in breakage at a distance. For FIDL code that
        // follow the linter naming conventions (lowercase library, CamelCase decl),
        // this will never come up in practice.

        let identifier = &reference.raw_sourced().unwrap().identifier;
        let components = identifier.to_vec();

        let lookup = Lookup::new(self, reference);

        match components.len() {
            1 => {
                let local_lookup = lookup.try_decl(&self.ctx.library, &components[0]);
                let root_loookup = lookup.try_decl(&self.ctx.all_libraries.borrow().root_library(), &components[0]);

                if local_lookup.is_some() {
                    return reference.set_key(local_lookup.unwrap());
                } else if root_loookup.is_some() {
                    return reference.set_key(root_loookup.unwrap());
                } else if context.allow_contextual {
                    return reference.mark_contextual();
                } else {
                    self.ctx.diagnostics.push_error(DiagnosticsError::new_name_not_found(
                        reference.span.as_ref().unwrap().clone(),
                        &components[0],
                        &self.ctx.library.name.get().expect("library name"),
                    ));
                }
            }
            2 => {
                if let Some(key) = lookup.try_decl(&self.ctx.library, &components[0]) {
                    return reference.set_key(key.member(&components[1]));
                } else if let Some(dep_library) = lookup.try_library(&vec![components[0].clone()]) {
                    if let Some(key) = lookup.must_decl(&dep_library, &components[1]) {
                        return reference.set_key(key);
                    }
                } else {
                    self.ctx.diagnostics.push_error(DiagnosticsError::new_name_not_found(
                        reference.span.as_ref().unwrap().clone(),
                        &components[0],
                        &self.ctx.library.name.get().expect("library name"),
                    ));
                }
            }
            _ => {
                let long_library_name = components[0..components.len() - 1].to_vec();
                let short_library_name = components[0..components.len() - 2].to_vec();

                if let Some(dep_library) = lookup.try_library(&long_library_name) {
                    if let Some(key) = lookup.must_decl(&dep_library, &components.last().unwrap()) {
                        return reference.set_key(key);
                    }
                } else if let Some(dep_library) = lookup.try_library(&short_library_name) {
                    if let Some(key) = lookup.must_decl(&dep_library, &components[components.len() - 2]) {
                        return reference.set_key(key.member(&components.last().unwrap()));
                    }
                } else {
                    self.ctx
                        .diagnostics
                        .push_error(DiagnosticsError::new_unknown_dependent_library(
                            reference.span.as_ref().unwrap().clone(),
                            &long_library_name,
                            &short_library_name,
                        ));
                }
            }
        }
    }

    fn is_resolved_to_head(&self, reference: &ast::Reference) -> bool {
        false
    }

    /// Inserts edges into graph_ for a parsed reference.
    fn insert_reference_edges(&self, reference: &ast::Reference, context: &ResolveContext) {
        match *reference.state.borrow() {
            ast::ReferenceState::Contextual(_) => return,
            ast::ReferenceState::Failed => return,
            ast::ReferenceState::Resolved(_) => return,
            _ => {}
        }

        let key = reference.key().unwrap();
        let decls = key.library.declarations.borrow();

        // Only insert edges if the target is in the same platform.
        //if (key.library.platform.value() != self.ctx.library.platform.value()) {
        //    return;
        //}

        // Note: key.library may is not necessarily library(), thus
        // key.library->declarations could be pre-decomposition or post-decomposition.
        // Although no branching is needed here, this is important to keep in mind.
        if let Some(decls) = decls.all.get_vec(&key.decl_name) {
            for decl in decls {
                let target: ast::Element = decl.clone().into();
                let enclosing = context.enclosing.clone();

                // Don't insert a self-loop.
                if target == enclosing {
                    continue;
                }

                let mut graph = self.graph.borrow_mut();

                // Only insert an edge if we have a chance of resolving to this target
                // post-decomposition (as opposed to one of the other same-named targets).
                //if (VersionSet::Intersect(target.availability.set(), enclosing.availability.set())) {
                // mut_graph.get_mut(&target).unwrap().neighbors.insert(enclosing);
                //}
            }
        }
    }

    fn resolve_reference(&self, reference: &ast::Reference, context: &ResolveContext) {
        let initial_state = reference.state.clone();
        let checkpoint = self.ctx.diagnostics.checkpoint();

        match *initial_state.borrow() {
            ast::ReferenceState::Failed => {}
            ast::ReferenceState::Resolved(_) => {
                // Nothing to do, either failed parsing or already attempted resolving.
                return;
            }
            ast::ReferenceState::Contextual(_) => self.resolve_contextual_reference(reference, context),
            ast::ReferenceState::Key(_) => self.resolve_key_reference(reference, context),
            _ => panic!("unexpected reference state"),
        }

        if reference.state == initial_state {
            // assert!(checkpoint.num_new_errors() > 0, "should have reported an error");
            // reference.mark_failed();
        }
    }

    fn resolve_contextual_reference(&self, reference: &ast::Reference, context: &ResolveContext) {}

    fn resolve_key_reference(&self, reference: &ast::Reference, context: &ResolveContext) {
        let decl = self.lookup_decl_by_key(reference, context);

        println!("{:?}", decl);

        if decl.is_none() {
            return;
        }

        let decl = decl.expect("early return if none");
        let ref_key = reference.key().unwrap();

        if ref_key.member_name.is_none() {
            reference.resolve_to(ast::Target::new(decl));
            return;
        }

        let lookup = Lookup::new(self, reference);
        let member = lookup.must_member(decl.clone(), &ref_key.member_name.unwrap());
        if member.is_none() {
            return;
        }

        reference.resolve_to(ast::Target::new_member(member.unwrap(), decl));
    }

    fn lookup_decl_by_key(&self, reference: &ast::Reference, context: &ResolveContext) -> Option<Declaration> {
        let key = reference.key().unwrap();
        let platform = key.library.platform;

        let declarations = key.library.declarations.borrow();
        let iter = declarations.all.get_vec(&key.decl_name).expect("key must exist");

        // Case #1: source and target libraries are versioned in the same platform.
        if self.ctx.library.platform == platform {
            for decl in iter {
                let us = context.enclosing.availability().range();
                let them = decl.availability().range();

                if let Some(overlap) = ast::VersionRange::intersect(Some(us), Some(them)) {
                    assert!(overlap == us, "referencee must outlive referencer");
                    return Some(decl.clone());
                } else {
                    println!("no overlap");
                }
            }

            // TODO(https://fxbug.dev/67858): Provide a nicer error message in the case where a
            // decl with that name does exist, but in a different version range.
            self.ctx.diagnostics.push_error(DiagnosticsError::new_name_not_found(
                reference.span.clone().unwrap(),
                &key.decl_name,
                &key.library.name.get().unwrap().clone(),
            ));

            return None;
        }

        // Case #2: source and target libraries are versioned in different platforms.
        let version = self
            .ctx
            .version_selection
            .lookup(platform.expect("runs after availaiblty step"));

        for decl in iter {
            if decl.availability().range().contains(version) {
                return Some(decl.clone());
            }
        }

        // TODO(https://fxbug.dev/67858): Provide a nicer error message in the case where a
        // decl with that name does exist, but in a different version range.
        self.ctx.diagnostics.push_error(DiagnosticsError::new_name_not_found(
            reference.span.clone().unwrap(),
            &key.decl_name,
            &key.library.name.get().unwrap().clone(),
        ));

        None
    }

    fn validate_reference(&self, reference: &ast::Reference, context: &ResolveContext) {}
}
