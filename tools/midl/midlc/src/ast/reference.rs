use std::{cell::RefCell, rc::Rc};

use super::{CompoundIdentifier, Declaration, Element, Identifier, Library, Name, Span};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target {
    pub target: Element,
    pub maybe_parent: Option<Declaration>,
}

impl Target {
    pub(crate) fn new(decl: Declaration) -> Self {
        Self {
            target: decl.into(),
            maybe_parent: None,
        }
    }

    pub(crate) fn new_member(member: Element, parent: Declaration) -> Self {
        Self {
            target: member,
            maybe_parent: Some(parent),
        }
    }

    pub fn element_or_parent_decl(&self) {}

    pub fn element(&self) -> Element {
        self.target.clone()
    }

    pub fn name(self) -> Name {
        match self.target {
            Element::Bits |
            Element::Builtin{..}|
            Element::Const{..}|
            Element::Enum {..}|
            Element::NewType|
            Element::Protocol{..}|
            Element::Resource|
            // Element::Service|
            Element::Struct{..}|
            Element::Table{..}|
            Element::Alias{..}|
            Element::Union{..}|
            Element::Overlay => {
                self.target.as_decl().unwrap().name()
            }
            //Element::BitsMember => {
            //    return self.maybe_parent.name.WithMemberName(std::string(static_cast<Bits::Member*>(target_)->name.data()));
            //}
            Element::EnumMember{ inner } => {
                let member = inner.borrow();
                return self.maybe_parent.unwrap().name().with_member_name(member.name.value.clone());  
            }
            Element::Library|
            //Element::ProtocolCompose|
            Element::ProtocolMethod{..} |
            Element::ResourceProperty|
            //Element::ServiceMember|
            Element::StructMember{..} |
            Element::TableMember {..}|
            // Element::OverlayMember
            Element::UnionMember{..} => panic!("invalid element kind")
        }
    }
}

/// String components that make up a sourced reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawSourced {
    pub identifier: CompoundIdentifier,
}

/// The initial, pre-decomposition decl that a synthetic reference points to.
/// This is distinct from the final, post-decomposition resolved target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawSynthetic {
    pub target: Target,
}

/// A key identifies a family of elements with a particular name. Unlike
/// RawSourced, the roles of each component have been decided, and the library
/// has been resolved. Unlike RawSynthetic, the key is stable across
/// decomposition, i.e. we can choose it before decomposing the AST, and then
/// use it for lookups after decomposing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferenceKey {
    pub library: Rc<Library>,
    pub decl_name: String,
    pub member_name: Option<String>,
}

impl ReferenceKey {
    pub fn member(&self, member_name: &String) -> Self {
        Self {
            library: self.library.clone(),
            decl_name: self.decl_name.clone(),
            member_name: Some(member_name.clone()),
        }
    }
}

/// An alternative to Key for a single component whose meaning is contextual.
/// For example, in fx.Handle:CHANNEL, CHANNEL is contextual and ultimately
/// resolves to fx.ObjType.CHANNEL.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Contextual {
    name: Identifier,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReferenceState {
    /// Initial state of a sourced reference.
    RawSourced(RawSourced),
    /// Initial state of a synthetic reference.
    RawSynthetic(RawSynthetic),
    /// Intermediate state for all references.
    Key(ReferenceKey),
    /// Alternative intermediate state for sourced references.
    Contextual(Contextual),
    /// Final state for valid references.
    Resolved(Target),
    /// Final state for invalid references.
    Failed,
}

impl std::fmt::Debug for ReferenceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceState::RawSourced(_) => write!(f, "RawSourced"),
            ReferenceState::RawSynthetic(_) => write!(f, "RawSynthetic"),
            ReferenceState::Key(val) => write!(
                f,
                "Key({}.{}/{:?})",
                val.library.name.get().unwrap().join("."),
                val.decl_name,
                val.member_name
            ),
            ReferenceState::Contextual(_) => write!(f, "Contextual"),
            ReferenceState::Resolved(_) => write!(f, "Resolved"),
            ReferenceState::Failed => write!(f, "Failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reference {
    pub state: RefCell<ReferenceState>,
    pub span: Option<Span>,
}

impl Reference {
    pub fn new_sourced(name: CompoundIdentifier) -> Self {
        assert!(!name.components.is_empty(), "expected at least one component");

        Self {
            span: Some(name.span.clone()),
            state: RefCell::from(ReferenceState::RawSourced(RawSourced { identifier: name })),
        }
    }

    pub fn new_synthetic(target: Target) -> Self {
        Self {
            span: None,
            state: RefCell::from(ReferenceState::RawSynthetic(RawSynthetic { target })),
        }
    }

    pub(crate) fn is_synthetic(&self) -> bool {
        self.span.is_none()
    }

    pub(crate) fn resolve_to(&self, target: Target) {
        {
            let state = self.state.borrow();
            assert!(
                matches!(*state, ReferenceState::Key(_) | ReferenceState::Contextual(_)),
                "invalid state"
            );
        }

        self.state.replace(ReferenceState::Resolved(target));
    }

    pub fn raw_synthetic(&self) -> Option<RawSynthetic> {
        match *self.state.borrow() {
            ReferenceState::RawSynthetic(ref raw) => Some(raw.clone()),
            _ => None,
        }
    }

    pub fn raw_sourced(&self) -> Option<RawSourced> {
        match *self.state.borrow() {
            ReferenceState::RawSourced(ref raw) => Some(raw.clone()),
            _ => None,
        }
    }

    pub fn resolved(&self) -> Option<Target> {
        match *self.state.borrow() {
            ReferenceState::Resolved(ref target) => Some(target.clone()),
            _ => None,
        }
    }

    pub fn key(&self) -> Option<ReferenceKey> {
        match *self.state.borrow() {
            ReferenceState::Key(ref key) => Some(key.clone()),
            _ => None,
        }
    }

    pub fn set_key(&self, key: ReferenceKey) {
        {
            let state = self.state.borrow();

            assert!(
                matches!(*state, ReferenceState::RawSourced(_) | ReferenceState::RawSynthetic(_)),
                "invalid state"
            );
        }

        self.state.replace(ReferenceState::Key(key));
    }

    pub fn mark_contextual(&self) {
        let raw = self.raw_sourced().expect("raw sourced reference");
        assert!(raw.identifier.components.len() == 1, "contextual requires 1 component");

        self.state.replace(ReferenceState::Contextual(Contextual {
            name: raw.identifier.components[0].clone(),
        }));
    }

    pub fn mark_failed(&self) {
        {
            let state = self.state.borrow();

            assert!(
                matches!(
                    *state,
                    ReferenceState::RawSourced(_) | ReferenceState::Key(_) | ReferenceState::Contextual(_)
                ),
                "invalid state"
            );
        }

        self.state.replace(ReferenceState::Failed);
    }
}
