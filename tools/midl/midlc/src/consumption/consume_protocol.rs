use std::cell::RefCell;
use std::rc::Rc;

use super::helpers::consume_catch_all;

use super::{consume_identifier, identifier_type_for_decl};
use super::{helpers::Pair, Rule};

use crate::ast::{self, Name, Span, TypeConstructor};
use crate::ast::{CompoundIdentifier, Declaration};
use crate::compiler::ParsingContext;
use crate::consumption::consume_attribute_list;
use crate::consumption::consume_comments::{consume_comment_block, consume_trailing_comment};
use crate::consumption::consume_type::consume_type_constructor;
use crate::consumption::helpers::consume_ordinal64;
use crate::diagnotics::DiagnosticsError;

fn consume_parameter_list(
    pair: Pair<'_>,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> TypeConstructor {
    assert!(pair.as_rule() == Rule::parameter_list);

    let mut maybe_type_ctor = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::PARENT_OPEN | Rule::PARENT_CLOSE => {}
            Rule::type_constructor => {
                maybe_type_ctor = Some(consume_type_constructor(current, &name_context, ctx));
            }
            _ => consume_catch_all(&current, "parameter list"),
        }
    }

    maybe_type_ctor.unwrap()
}

fn consume_protocol_request() -> Option<TypeConstructor> {
    None
}

fn consume_protocol_response() -> Option<TypeConstructor> {
    None
}

fn create_method_result(
    success_variant_context: &Rc<ast::NamingContext>,
    err_variant_context: &Rc<ast::NamingContext>,
    framework_err_variant_context: &Rc<ast::NamingContext>,
    has_err: bool,
    has_framework_err: bool,
    response_span: Span,
    maybe_error_ctor: Option<Pair<'_>>,
    success_variant_ctor: TypeConstructor,
    ctx: &mut ParsingContext<'_>,
) -> ast::TypeConstructor {
    // let ordinal_source = SourceElement(Token(), Token());
    let mut result_members = vec![];

    enum Result {
        SuccessOrdinal = 1,
        ErrorOrdinal = 2,
        FrameworkErrorOrdinal = 3,
    }

    result_members.push(Rc::new(RefCell::new(ast::UnionMember {
        attributes: ast::AttributeList(vec![]),
        documentation: None,
        ordinal: ast::RawOrdinal64 {
            value: 1, // Result::Success,
            span: ast::Span::empty(),
        },
        maybe_used: Some(ast::UnionMemberUsed {
            name: success_variant_context.name(),
            type_ctor: success_variant_ctor,
        }),
        span: ast::Span::empty(),
    })));

    if has_err {
        // Compile the error type.
        let error_type_ctor = consume_type_constructor(maybe_error_ctor.unwrap(), err_variant_context, ctx);
        // assert!(error_type_ctor != nullptr, "missing err type ctor");

        //result_members.emplace_back(ConsumeOrdinal(RawOrdinal64(ordinal_source, Result::ErrorOrdinal)),
        //    error_type_ctor, err_variant_context.name(), AttributeList::new());

        result_members.push(Rc::new(RefCell::new(ast::UnionMember {
            attributes: ast::AttributeList(vec![]),
            documentation: None,
            ordinal: ast::RawOrdinal64 {
                value: 2, // Result::ErrorOrdinal,
                span: ast::Span::empty(),
            },
            maybe_used: Some(ast::UnionMemberUsed {
                name: err_variant_context.name(),
                type_ctor: error_type_ctor,
            }),
            span: ast::Span::empty(),
        })));
    } else {
        // If there's no error, the error variant is reserved.
        // result_members.emplace_back(Union::Member::Reserved( ConsumeOrdinal(RawOrdinal64(ordinal_source, kErrorOrdinal)), err_variant_context.name(), std::make_unique<AttributeList>()));

        result_members.push(Rc::new(RefCell::new(ast::UnionMember {
            attributes: ast::AttributeList(vec![]),
            documentation: None,
            ordinal: ast::RawOrdinal64 {
                value: 2, // Result::ErrorOrdinal,
                span: ast::Span::empty(),
            },
            maybe_used: None,
            span: ast::Span::empty(),
        })));
    }

    if has_framework_err {
        let error_type_ctor = identifier_type_for_decl(ctx.framework_err_type.clone());
        // assert!(error_type_ctor.is_none(), "missing framework_err type ctor");

        result_members.push(Rc::new(RefCell::new(ast::UnionMember {
            attributes: ast::AttributeList(vec![]),
            documentation: None,
            ordinal: ast::RawOrdinal64 {
                value: 3, // Result::FrameworkErrorOrdinal,
                span: ast::Span::empty(),
            },
            maybe_used: Some(ast::UnionMemberUsed {
                name: framework_err_variant_context.name(),
                type_ctor: error_type_ctor,
            }),
            span: ast::Span::empty(),
        })));
    }

    // framework_err is not defined if the method is not flexible.

    let result_context = err_variant_context.parent();
    let result_name = ast::Name::create_anonymous(
        ctx.library.clone(),
        response_span,
        result_context,
        ast::NameProvenance::GeneratedResultUnion,
    );

    let union_decl = ast::Union {
        name: result_name,
        span: ast::Span::empty(),
        members: result_members,
        attributes: ast::AttributeList(vec![]),
        documentation: None,
        strictness: ast::Strictness::Flexible,
        compiled: false,
        compiling: false,
        recursive: false,
    };

    let result_decl: ast::Declaration = union_decl.into();

    ctx.library.declarations.borrow_mut().insert(result_decl.clone());

    //if (!RegisterDecl(std::move(union_decl)))
    //return false;

    identifier_type_for_decl(result_decl)
}

fn consume_protocol_method(
    pair: Pair<'_>,
    protocol_name: Name,
    block_comment: Option<Pair<'_>>,
    protocol_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::ProtocolMethod, DiagnosticsError> {
    let pair_span = pair.as_span();
    let mut method_name = None;

    let attributes = Vec::new();
    let mut documentation = block_comment.and_then(consume_comment_block);
    let strictness = ast::Strictness::Flexible;

    let mut maybe_request = None;
    let mut maybe_response = None;

    let mut has_request = false;
    let mut has_response = false;
    let mut has_error = false;

    // NOTE: we need to first determine what parts are present on this method
    for current in pair.clone().into_inner() {
        match current.as_rule() {
            Rule::protocol_request => {
                has_request = true;
            }
            Rule::protocol_response => {
                has_response = true;

                for current in current.into_inner() {
                    match current.as_rule() {
                        Rule::type_constructor => {
                            has_error = true;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => method_name = Some(consume_identifier(&current, ctx)),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics))*/ }
            Rule::protocol_request => {
                let protocol_context = protocol_context.clone();

                maybe_request = Some(consume_parameter_list(
                    current.into_inner().next().unwrap(),
                    protocol_context.enter_request(method_name.clone().unwrap()),
                    ctx,
                ));
            }
            Rule::protocol_response => {
                let protocol_context = protocol_context.clone();

                // has_framework_error is true for flexible two-way methods. We already
                // checked has_response in the outer if block, so to see whether this is a
                // two-way method or an event, we check has_request here.
                let has_framework_error = has_request && strictness == ast::Strictness::Flexible;

                let mut inner = current.into_inner();
                let param_list_token = inner.next().unwrap();
                let optional_error_token = inner.next();

                let response_context = if has_request {
                    protocol_context.enter_response(method_name.as_ref().unwrap().clone())
                } else {
                    protocol_context.enter_event(method_name.as_ref().unwrap().clone())
                };

                if has_error || has_framework_error {
                    let response_span = param_list_token.as_span();
                    let response_span = ast::Span::from_pest(response_span, ctx.source_id);

                    let success_variant_context: Rc<ast::NamingContext>;
                    let err_variant_context: Rc<ast::NamingContext>;
                    let framework_err_variant_context: Rc<ast::NamingContext>;

                    let method_name = method_name.as_ref().unwrap().clone().data;
                    let protocol_name = protocol_name.decl_name();

                    // In protocol P, if method M is flexible or uses the error syntax, its
                    // response is the following compiler-generated union:
                    //
                    //     type P_M_Result = union {
                    //       // The "success variant".
                    //       1: response @generated_name("P_M_Response") [user specified response type];
                    //       // The "error variant". Marked `reserved` if there is no error.
                    //       2: err @generated_name("P_M_Error") [user specified error type];
                    //       // Omitted for strict methods.
                    //       3: framework_err fidl.FrameworkErr;
                    //     };
                    //
                    // This naming scheme is inconsistent with other compiler-generated
                    // names (e.g. PMRequest) because the error syntax predates anonymous
                    // layouts. We keep it like this for backwards compatibility.
                    //
                    // Note that although the success variant is named P_M_Response, in the
                    // midlc codebase we use the word "response" to refer to the outermost
                    // type, which in this case is P_M_Result.

                    response_context.set_name_override(
                        vec![protocol_name.clone(), method_name.clone(), "Result".to_string()].join("_"),
                    );

                    success_variant_context = response_context
                        .clone()
                        .enter_member(ast::Span::new_raw("response", ctx.source_id));
                    success_variant_context.set_name_override(
                        vec![protocol_name.clone(), method_name.clone(), "Response".to_string()].join("_"),
                    );

                    err_variant_context = response_context
                        .clone()
                        .enter_member(ast::Span::new_raw("err", ctx.source_id));
                    err_variant_context
                        .set_name_override(vec![protocol_name, method_name, "Error".to_string()].join("_"));

                    framework_err_variant_context =
                        response_context.enter_member(ast::Span::new_raw("framework_err", ctx.source_id));

                    let result_payload = consume_parameter_list(param_list_token, success_variant_context.clone(), ctx);

                    // ZX_ASSERT_MSG(err_variant_context != nullptr && framework_err_variant_context != nullptr, "error type contexts should have been computed");

                    let response_payload = create_method_result(
                        &success_variant_context,
                        &err_variant_context,
                        &framework_err_variant_context,
                        has_error,
                        has_framework_error,
                        response_span,
                        optional_error_token,
                        result_payload,
                        ctx,
                    );

                    maybe_response = Some(response_payload);
                } else {
                    let response_payload = consume_parameter_list(param_list_token, response_context, ctx);
                    maybe_response = Some(response_payload);
                }
            }
            Rule::trailing_comment => {
                documentation = match (documentation, consume_trailing_comment(current)) {
                    (c, None) | (None, c) => c,
                    (Some(existing), Some(new)) => Some(ast::Comment {
                        text: [existing.text, new.text].join("\n"),
                    }),
                };
            }
            _ => consume_catch_all(&current, "protocol method"),
        }
    }

    Ok(ast::ProtocolMethod {
        name: method_name.unwrap(),
        documentation,
        strictness,
        attributes,
        request_payload: None,
        response_payload: None,
        has_request,
        has_response,
        has_error,
        maybe_request,
        maybe_response,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    })
}

fn consume_compose(ctx: &mut ParsingContext<'_>) {}

pub(crate) fn consume_protocol_declaration(
    pair: Pair<'_>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Protocol, DiagnosticsError> {
    let pair_span = pair.as_span();

    let mut name = None;
    let mut name_context = None;

    let mut pending_field_comment = None;
    let mut methods = Vec::new();
    let mut attributes = None;
    let mut composes = Vec::new();

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::PROTOCOL_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);
                let sourced = Name::create_sourced(ctx.library.clone(), name_span);

                name_context = Some(ast::NamingContext::create(&sourced));
                name = Some(sourced);
            }
            Rule::block_attribute_list => {
                attributes = Some(consume_attribute_list(current, ctx));
            }
            Rule::protocol_method => {
                let name_context = name_context.as_ref().unwrap().clone();

                match consume_protocol_method(
                    current,
                    name.as_ref().unwrap().clone(),
                    pending_field_comment.take(),
                    name_context,
                    ctx,
                ) {
                    Ok(method) => {
                        methods.push(Rc::from(RefCell::new(method)));
                    }
                    Err(err) => ctx.diagnostics.push_error(err),
                }
            }
            Rule::protocol_event => {}
            Rule::protocol_compose => consume_compose(ctx),
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                Span::from_pest(current.as_span(), ctx.source_id),
            )),
            _ => consume_catch_all(&current, "protocol"),
        }
    }

    Ok(ast::Protocol {
        name: name.unwrap(),
        methods,
        composes,
        attributes: attributes.unwrap(),
        documentation: None,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
        compiled: false,
        compiling: false,
        recursive: false,
    })
}
