use std::{cell::RefCell, rc::Rc};

use super::{
    AttributeList, Comment, Declaration, Name, Span, TypeConstructor, WithAttributes, WithDocumentation, WithName,
    WithSpan,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceProperty {
    pub(crate) name: Span,

    /// The attributes of this resource property.
    pub(crate) attributes: AttributeList,

    pub(crate) type_ctor: TypeConstructor,
}

/// An resource declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Resource {
    /// The name of the resource.
    ///
    /// ```ignore
    /// resource Foo { .. }
    ///          ^^^
    /// ```
    pub(crate) name: Name,

    /// The attributes of this resource.
    ///
    /// ```ignore
    /// @available(added=1)
    /// ^^^^^^^^^^^^^^^^^^^
    /// resource Foo { .. }
    /// ```
    pub(crate) attributes: AttributeList,

    /// The documentation for this resource.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// resource Foo { .. }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The location of this resource in the text representation.
    pub(crate) span: Span,

    // Set during construction.
    pub(crate) subtype_ctor: TypeConstructor,
    pub(crate) properties: Vec<Rc<RefCell<ResourceProperty>>>,

    // Set during compilation
    pub(crate) compiled: bool,
    pub(crate) compiling: bool,
    pub(crate) recursive: bool,
}

impl Resource {
    pub(crate) fn lookup_property(&self, name: &str) -> Option<Rc<RefCell<ResourceProperty>>> {
        for property_original in self.properties.iter() {
            let property = property_original.borrow();

            if property.name.data == name {
                return Some(property_original.clone());
            }
        }

        None
    }
}

/// An opaque identifier for a field in an AST model. Use the
/// `model[field_id]` syntax to resolve the id to an `ast::Field`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourcePropertyId(pub(super) u32);

impl Resource {
    pub fn iter_properties(&self) -> impl ExactSizeIterator<Item = (ResourcePropertyId, &Rc<RefCell<ResourceProperty>>)> + Clone {
        self.properties
            .iter()
            .enumerate()
            .map(|(idx, field)| (ResourcePropertyId(idx as u32), field))
    }
}

impl Into<Declaration> for Resource {
    fn into(self) -> Declaration {
        Declaration::Resource {
            decl: Rc::new(RefCell::new(self)),
        }
    }
}

impl WithSpan for Resource {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Resource {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Resource {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}

impl WithName for Resource {
    fn name(&self) -> &Name {
        &self.name
    }
}
