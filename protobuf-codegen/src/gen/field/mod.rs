use protobuf::descriptor::field_descriptor_proto::Type;
use protobuf::descriptor::*;
use protobuf::reflect::ReflectValueRef;
use protobuf::reflect::RuntimeFieldType;
use protobuf::reflect::Syntax;
use protobuf::rt;
use protobuf::rt::WireType;
use protobuf_parse::ProtobufAbsPath;

use crate::customize::ctx::CustomizeElemCtx;
use crate::customize::rustproto_proto::customize_from_rustproto_for_field;
use crate::customize::Customize;
use crate::gen::code_writer::CodeWriter;
use crate::gen::code_writer::Visibility;
use crate::gen::file_and_mod::FileAndMod;
use crate::gen::inside::protobuf_crate_path;
use crate::gen::map::map_entry;
use crate::gen::message::RustTypeMessage;
use crate::gen::oneof::OneofField;
use crate::gen::protoc_insertion_point::write_protoc_insertion_point_for_field;
use crate::gen::rust;
use crate::gen::rust::EXPR_NONE;
use crate::gen::rust::EXPR_VEC_NEW;
use crate::gen::rust_name::RustIdent;
use crate::gen::rust_name::RustIdentWithPath;
use crate::gen::rust_types_values::*;
use crate::gen::scope::EnumValueWithContext;
use crate::gen::scope::FieldWithContext;
use crate::gen::scope::MessageOrEnumWithScope;
use crate::gen::scope::MessageWithScope;
use crate::gen::scope::RootScope;
use crate::gen::scope::WithScope;

mod accessor;

fn type_is_copy(field_type: field_descriptor_proto::Type) -> bool {
    match field_type {
        field_descriptor_proto::Type::TYPE_MESSAGE
        | field_descriptor_proto::Type::TYPE_STRING
        | field_descriptor_proto::Type::TYPE_BYTES => false,
        _ => true,
    }
}

trait FieldDescriptorProtoTypeExt {
    fn read(&self, is: &str, primitive_type_variant: PrimitiveTypeVariant) -> String;
    fn is_s_varint(&self) -> bool;
}

impl FieldDescriptorProtoTypeExt for field_descriptor_proto::Type {
    fn read(&self, is: &str, primitive_type_variant: PrimitiveTypeVariant) -> String {
        match *self {
            field_descriptor_proto::Type::TYPE_ENUM => format!("{}.read_enum_or_unknown()", is),
            _ => match primitive_type_variant {
                PrimitiveTypeVariant::Default => format!("{}.read_{}()", is, protobuf_name(*self)),
                PrimitiveTypeVariant::TokioBytes => {
                    let protobuf_name = match self {
                        field_descriptor_proto::Type::TYPE_STRING => "chars",
                        _ => protobuf_name(*self),
                    };
                    format!("{}.read_tokio_{}()", is, protobuf_name)
                }
            },
        }
    }

    /// True if self is signed integer with zigzag encoding
    fn is_s_varint(&self) -> bool {
        match *self {
            field_descriptor_proto::Type::TYPE_SINT32
            | field_descriptor_proto::Type::TYPE_SINT64 => true,
            _ => false,
        }
    }
}

fn type_protobuf_name(field_type: field_descriptor_proto::Type) -> &'static str {
    match field_type {
        Type::TYPE_INT32 => "int32",
        Type::TYPE_INT64 => "int64",
        Type::TYPE_UINT32 => "uint32",
        Type::TYPE_UINT64 => "uint64",
        Type::TYPE_SINT32 => "sint32",
        Type::TYPE_SINT64 => "sint64",
        Type::TYPE_BOOL => "bool",
        Type::TYPE_FIXED32 => "fixed32",
        Type::TYPE_FIXED64 => "fixed64",
        Type::TYPE_SFIXED32 => "sfixed32",
        Type::TYPE_SFIXED64 => "sfixed64",
        Type::TYPE_FLOAT => "float",
        Type::TYPE_DOUBLE => "double",
        Type::TYPE_STRING => "string",
        Type::TYPE_BYTES => "bytes",
        Type::TYPE_ENUM | Type::TYPE_MESSAGE | Type::TYPE_GROUP => panic!(),
    }
}

fn field_type_protobuf_name<'a>(field: &'a FieldDescriptorProto) -> &'a str {
    if field.has_type_name() {
        field.type_name()
    } else {
        type_protobuf_name(field.field_type())
    }
}

// size of value for type, None if variable
fn field_type_size(field_type: field_descriptor_proto::Type) -> Option<u32> {
    match field_type {
        field_descriptor_proto::Type::TYPE_BOOL => Some(1),
        t if WireType::for_type(t) == WireType::Fixed32 => Some(4),
        t if WireType::for_type(t) == WireType::Fixed64 => Some(8),
        _ => None,
    }
}

/// Optional fields can be stored are `Option<T>` or `SingularPtrField<T>`.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum OptionKind {
    /// Field is `Option<T>`
    Option,
    /// Field is `SingularPtrField<T>`
    MessageField,
}

impl OptionKind {
    fn wrap_element(&self, element_type: RustType) -> RustType {
        let element_type = Box::new(element_type);
        match self {
            OptionKind::Option => RustType::Option(element_type),
            OptionKind::MessageField => RustType::MessageField(element_type),
        }
    }

    // Type of `as_ref()` operation
    fn as_ref_type(&self, element_type: RustType) -> RustType {
        match self {
            OptionKind::Option => RustType::Option(Box::new(element_type.ref_type())),
            OptionKind::MessageField => RustType::MessageField(Box::new(element_type.ref_type())),
        }
    }

    fn _as_option_ref(&self, v: &str) -> String {
        match self {
            OptionKind::Option | OptionKind::MessageField => format!("{}.as_ref()", v),
        }
    }

    fn unwrap_or_else(&self, what: &str, default_value: &str) -> String {
        match self {
            _ => format!("{}.unwrap_or_else(|| {})", what, default_value),
        }
    }

    fn unwrap_ref_or_else(&self, what: &str, default_value: &str) -> String {
        match self {
            _ => format!("{}.unwrap_or_else(|| {})", what, default_value),
        }
    }

    fn wrap_value(&self, value: &str, customize: &Customize) -> String {
        match self {
            OptionKind::Option => format!("::std::option::Option::Some({})", value),
            OptionKind::MessageField => format!(
                "{}::MessageField::some({})",
                protobuf_crate_path(customize),
                value
            ),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum SingularFieldFlag {
    // proto2 or proto3 message
    WithFlag {
        required: bool,
        option_kind: OptionKind,
    },
    // proto3
    WithoutFlag,
}

impl SingularFieldFlag {
    pub fn is_required(&self) -> bool {
        match *self {
            SingularFieldFlag::WithFlag { required, .. } => required,
            SingularFieldFlag::WithoutFlag => false,
        }
    }
}

#[derive(Clone)]
pub(crate) struct SingularField<'a> {
    pub flag: SingularFieldFlag,
    pub elem: FieldElem<'a>,
}

impl<'a> SingularField<'a> {
    fn rust_storage_type(&self, reference: &FileAndMod) -> RustType {
        match self.flag {
            SingularFieldFlag::WithFlag { option_kind, .. } => {
                option_kind.wrap_element(self.elem.rust_storage_elem_type(reference))
            }
            SingularFieldFlag::WithoutFlag => self.elem.rust_storage_elem_type(reference),
        }
    }

    fn default_value(
        &self,
        customize: &Customize,
        reference: &FileAndMod,
        const_expr: bool,
    ) -> String {
        self.rust_storage_type(reference)
            .default_value(customize, const_expr)
    }
}

/// Repeated field can be `Vec<T>` or `RepeatedField<T>`.
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum RepeatedFieldKind {
    Vec,
}

impl RepeatedFieldKind {
    fn wrap_element(&self, element_type: RustType) -> RustType {
        let element_type = Box::new(element_type);
        match self {
            RepeatedFieldKind::Vec => RustType::Vec(element_type),
        }
    }

    fn default(&self) -> String {
        match self {
            RepeatedFieldKind::Vec => EXPR_VEC_NEW.to_owned(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct RepeatedField<'a> {
    pub elem: FieldElem<'a>,
    pub packed: bool,
}

impl<'a> RepeatedField<'a> {
    fn kind(&self) -> RepeatedFieldKind {
        RepeatedFieldKind::Vec
    }

    fn rust_type(&self, reference: &FileAndMod) -> RustType {
        self.kind()
            .wrap_element(self.elem.rust_storage_elem_type(reference))
    }

    fn default(&self) -> String {
        self.kind().default()
    }
}

#[derive(Clone)]
pub struct MapField<'a> {
    _message: MessageWithScope<'a>,
    key: FieldElem<'a>,
    value: FieldElem<'a>,
}

#[derive(Clone)]
pub(crate) enum FieldKind<'a> {
    // optional or required
    Singular(SingularField<'a>),
    // repeated except map
    Repeated(RepeatedField<'a>),
    // map
    Map(MapField<'a>),
    // part of oneof
    Oneof(OneofField<'a>),
}

impl<'a> FieldKind<'a> {
    pub fn default(
        &self,
        customize: &Customize,
        reference: &FileAndMod,
        const_expr: bool,
    ) -> String {
        match self {
            FieldKind::Singular(s) => s.default_value(customize, reference, const_expr),
            FieldKind::Repeated(r) => r.default(),
            FieldKind::Oneof(..) => EXPR_NONE.to_owned(),
            FieldKind::Map(..) => panic!("map fields cannot have field value"),
        }
    }
}

#[derive(Clone)]
pub(crate) enum SingularOrOneofField<'a> {
    Singular(SingularField<'a>),
    Oneof(OneofField<'a>),
}

impl<'a> SingularOrOneofField<'a> {
    fn elem(&self) -> &FieldElem {
        match self {
            SingularOrOneofField::Singular(SingularField { ref elem, .. }) => elem,
            SingularOrOneofField::Oneof(OneofField { ref elem, .. }) => elem,
        }
    }

    // Type of `xxx` function for singular type.
    pub fn getter_return_type(&self, reference: &FileAndMod) -> RustType {
        let elem = self.elem();
        if let FieldElem::Enum(ref en) = elem {
            en.enum_rust_type(reference)
        } else if elem.is_copy() {
            elem.rust_storage_elem_type(reference)
        } else {
            elem.rust_storage_elem_type(reference).ref_type()
        }
    }
}

// Representation of map entry: key type and value type
#[derive(Clone, Debug)]
pub struct EntryKeyValue<'a>(FieldElem<'a>, FieldElem<'a>);

#[derive(Clone, Debug)]
pub(crate) struct FieldElemEnum<'a> {
    /// Enum default value variant, either from proto or from enum definition
    default_value: EnumValueWithContext<'a>,
}

impl<'a> FieldElemEnum<'a> {
    fn rust_name_relative(&self, reference: &FileAndMod) -> RustIdentWithPath {
        message_or_enum_to_rust_relative(&self.default_value.en, reference)
    }

    fn enum_rust_type(&self, reference: &FileAndMod) -> RustType {
        RustType::Enum(
            self.rust_name_relative(reference),
            self.default_value.rust_name(),
            self.default_value.proto.proto().number(),
        )
    }

    fn enum_or_unknown_rust_type(&self, reference: &FileAndMod) -> RustType {
        RustType::EnumOrUnknown(
            self.rust_name_relative(reference),
            self.default_value.rust_name(),
            self.default_value.proto.proto().number(),
        )
    }

    fn default_value_rust_expr(&self, reference: &FileAndMod) -> RustIdentWithPath {
        self.rust_name_relative(reference)
            .to_path()
            .with_ident(self.default_value.rust_name())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FieldElemMessage<'a> {
    pub message: MessageWithScope<'a>,
}

impl<'a> FieldElemMessage<'a> {
    fn rust_name_relative(&self, reference: &FileAndMod) -> RustTypeMessage {
        RustTypeMessage(message_or_enum_to_rust_relative(&self.message, reference))
    }

    fn rust_type(&self, reference: &FileAndMod) -> RustType {
        RustType::Message(self.rust_name_relative(reference))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FieldElem<'a> {
    Primitive(field_descriptor_proto::Type, PrimitiveTypeVariant),
    Message(FieldElemMessage<'a>),
    Enum(FieldElemEnum<'a>),
    Group,
}

impl<'a> FieldElem<'a> {
    fn proto_type(&self) -> field_descriptor_proto::Type {
        match *self {
            FieldElem::Primitive(t, ..) => t,
            FieldElem::Group => field_descriptor_proto::Type::TYPE_GROUP,
            FieldElem::Message(..) => field_descriptor_proto::Type::TYPE_MESSAGE,
            FieldElem::Enum(..) => field_descriptor_proto::Type::TYPE_ENUM,
        }
    }

    fn is_copy(&self) -> bool {
        type_is_copy(self.proto_type())
    }

    pub fn rust_storage_elem_type(&self, reference: &FileAndMod) -> RustType {
        match *self {
            FieldElem::Primitive(t, PrimitiveTypeVariant::Default) => rust_name(t),
            FieldElem::Primitive(
                field_descriptor_proto::Type::TYPE_STRING,
                PrimitiveTypeVariant::TokioBytes,
            ) => RustType::Chars,
            FieldElem::Primitive(
                field_descriptor_proto::Type::TYPE_BYTES,
                PrimitiveTypeVariant::TokioBytes,
            ) => RustType::Bytes,
            FieldElem::Primitive(.., PrimitiveTypeVariant::TokioBytes) => unreachable!(),
            FieldElem::Group => RustType::Group,
            FieldElem::Message(ref m) => m.rust_type(reference),
            FieldElem::Enum(ref en) => en.enum_or_unknown_rust_type(reference),
        }
    }

    // Type of set_xxx function parameter type for singular fields
    pub fn rust_set_xxx_param_type(&self, reference: &FileAndMod) -> RustType {
        if let FieldElem::Enum(ref en) = *self {
            en.enum_rust_type(reference)
        } else {
            self.rust_storage_elem_type(reference)
        }
    }

    fn protobuf_type_gen(&self, reference: &FileAndMod) -> ProtobufTypeGen {
        match *self {
            FieldElem::Primitive(t, v) => ProtobufTypeGen::Primitive(t, v),
            FieldElem::Message(ref m) => ProtobufTypeGen::Message(m.rust_name_relative(reference)),
            FieldElem::Enum(ref en) => {
                ProtobufTypeGen::EnumOrUnknown(en.rust_name_relative(reference))
            }
            FieldElem::Group => unreachable!(),
        }
    }

    /// implementation of ProtobufType trait
    fn lib_protobuf_type(&self, reference: &FileAndMod) -> String {
        self.protobuf_type_gen(reference)
            .rust_type(&reference.customize)
    }

    fn primitive_type_variant(&self) -> PrimitiveTypeVariant {
        match self {
            &FieldElem::Primitive(_, v) => v,
            _ => PrimitiveTypeVariant::Default,
        }
    }
}

fn field_elem<'a>(
    field: &FieldWithContext,
    root_scope: &'a RootScope<'a>,
    customize: &Customize,
) -> FieldElem<'a> {
    if let RuntimeFieldType::Map(..) = field.field.runtime_field_type() {
        unreachable!();
    }

    if field.field.proto().field_type() == field_descriptor_proto::Type::TYPE_GROUP {
        FieldElem::Group
    } else if field.field.proto().has_type_name() {
        let message_or_enum = root_scope
            .find_message_or_enum(&ProtobufAbsPath::from(field.field.proto().type_name()));
        match (field.field.proto().field_type(), message_or_enum) {
            (
                field_descriptor_proto::Type::TYPE_MESSAGE,
                MessageOrEnumWithScope::Message(message),
            ) => FieldElem::Message(FieldElemMessage {
                message: message.clone(),
            }),
            (
                field_descriptor_proto::Type::TYPE_ENUM,
                MessageOrEnumWithScope::Enum(enum_with_scope),
            ) => {
                let default_value = if field.field.proto().has_default_value() {
                    enum_with_scope.value_by_name(field.field.proto().default_value())
                } else {
                    enum_with_scope.values()[0].clone()
                };
                FieldElem::Enum(FieldElemEnum { default_value })
            }
            _ => panic!("unknown named type: {:?}", field.field.proto().field_type()),
        }
    } else if field.field.proto().has_field_type() {
        let tokio_for_bytes = customize.tokio_bytes_for_bytes.unwrap_or(false);
        let tokio_for_string = customize.tokio_bytes_for_string.unwrap_or(false);

        let elem = match field.field.proto().field_type() {
            field_descriptor_proto::Type::TYPE_STRING if tokio_for_string => FieldElem::Primitive(
                field_descriptor_proto::Type::TYPE_STRING,
                PrimitiveTypeVariant::TokioBytes,
            ),
            field_descriptor_proto::Type::TYPE_BYTES if tokio_for_bytes => FieldElem::Primitive(
                field_descriptor_proto::Type::TYPE_BYTES,
                PrimitiveTypeVariant::TokioBytes,
            ),
            t => FieldElem::Primitive(t, PrimitiveTypeVariant::Default),
        };

        elem
    } else {
        panic!(
            "neither type_name, nor field_type specified for field: {}",
            field.field.name()
        );
    }
}

#[derive(Clone)]
pub(crate) struct FieldGen<'a> {
    _root_scope: &'a RootScope<'a>,
    syntax: Syntax,
    pub proto_field: FieldWithContext<'a>,
    // field name in generated code
    pub rust_name: RustIdent,
    pub proto_type: field_descriptor_proto::Type,
    wire_type: WireType,
    pub kind: FieldKind<'a>,
    pub expose_field: bool,
    pub generate_accessors: bool,
    pub generate_getter: bool,
    customize: Customize,
    path: Vec<i32>,
    info: Option<&'a SourceCodeInfo>,
}

impl<'a> FieldGen<'a> {
    pub fn parse(
        field: FieldWithContext<'a>,
        root_scope: &'a RootScope<'a>,
        parent_customize: &CustomizeElemCtx<'a>,
        path: Vec<i32>,
        info: Option<&'a SourceCodeInfo>,
    ) -> FieldGen<'a> {
        let customize = parent_customize
            .child(
                &customize_from_rustproto_for_field(field.field.proto().options.get_or_default()),
                &field.field,
            )
            .for_elem;

        let syntax = field.message.scope.file_scope.syntax();

        let field_may_have_custom_default_value = syntax == Syntax::Proto2
            && field.field.proto().label() != field_descriptor_proto::Label::LABEL_REPEATED
            && field.field.proto().field_type() != field_descriptor_proto::Type::TYPE_MESSAGE;

        let default_expose_field = !field_may_have_custom_default_value;
        let expose_field = customize.expose_fields.unwrap_or(default_expose_field);

        let default_generate_accessors = !expose_field;
        let generate_accessors = customize
            .generate_accessors
            .unwrap_or(default_generate_accessors)
            || field.is_oneof();

        let default_generate_getter = generate_accessors || field_may_have_custom_default_value;
        let generate_getter =
            customize.generate_getter.unwrap_or(default_generate_getter) || field.is_oneof();

        let kind = match field.field.runtime_field_type() {
            RuntimeFieldType::Map(..) => {
                let message = root_scope
                    .find_message(&ProtobufAbsPath::from(field.field.proto().type_name()));

                let (key, value) = map_entry(&message).unwrap();

                let key = field_elem(&key, root_scope, &customize);
                let value = field_elem(&value, root_scope, &customize);

                FieldKind::Map(MapField {
                    _message: message,
                    key,
                    value,
                })
            }
            RuntimeFieldType::Repeated(..) => {
                let elem = field_elem(&field, root_scope, &customize);

                FieldKind::Repeated(RepeatedField {
                    elem,
                    packed: field.field.proto().options.get_or_default().packed(),
                })
            }
            RuntimeFieldType::Singular(..) => {
                let elem = field_elem(&field, root_scope, &customize);

                if let Some(oneof) = field.oneof() {
                    FieldKind::Oneof(OneofField::parse(&oneof, &field.field, elem, root_scope))
                } else {
                    let flag = if field.message.scope.file_scope.syntax() == Syntax::Proto3
                        && field.field.proto().field_type()
                            != field_descriptor_proto::Type::TYPE_MESSAGE
                    {
                        SingularFieldFlag::WithoutFlag
                    } else {
                        let required = field.field.proto().label()
                            == field_descriptor_proto::Label::LABEL_REQUIRED;
                        let option_kind = match field.field.proto().field_type() {
                            field_descriptor_proto::Type::TYPE_MESSAGE => OptionKind::MessageField,
                            _ => OptionKind::Option,
                        };

                        SingularFieldFlag::WithFlag {
                            required,
                            option_kind,
                        }
                    };
                    FieldKind::Singular(SingularField { elem, flag })
                }
            }
        };

        FieldGen {
            _root_scope: root_scope,
            syntax: field.message.scope().file_scope.syntax(),
            rust_name: rust_field_name_for_protobuf_field_name(&field.field.name()),
            proto_type: field.field.proto().field_type(),
            wire_type: WireType::for_type(field.field.proto().field_type()),
            proto_field: field,
            kind,
            expose_field,
            generate_accessors,
            generate_getter,
            customize,
            path,
            info,
        }
    }

    // for message level
    fn file_and_mod(&self) -> FileAndMod {
        self.proto_field
            .message
            .scope
            .file_and_mod(self.customize.clone())
    }

    fn tag_size(&self) -> u32 {
        rt::tag_size(self.proto_field.number()) as u32
    }

    fn is_singular(&self) -> bool {
        match self.kind {
            FieldKind::Singular(..) => true,
            _ => false,
        }
    }

    fn is_repeated_or_map(&self) -> bool {
        match self.kind {
            FieldKind::Repeated(..) | FieldKind::Map(..) => true,
            _ => false,
        }
    }

    fn is_repeated_packed(&self) -> bool {
        match self.kind {
            FieldKind::Repeated(RepeatedField { packed: true, .. }) => true,
            _ => false,
        }
    }

    fn map(&self) -> &MapField {
        match self.kind {
            FieldKind::Map(ref map) => &map,
            _ => panic!("not a map field: {}", self.reconstruct_def()),
        }
    }

    // TODO: drop it
    pub fn elem(&self) -> &FieldElem {
        match self.kind {
            FieldKind::Singular(SingularField { ref elem, .. }) => &elem,
            FieldKind::Repeated(RepeatedField { ref elem, .. }) => &elem,
            FieldKind::Oneof(OneofField { ref elem, .. }) => &elem,
            FieldKind::Map(..) => unreachable!(),
        }
    }

    // type of field in struct
    pub fn full_storage_type(&self, reference: &FileAndMod) -> RustType {
        match self.kind {
            FieldKind::Repeated(ref repeated) => repeated.rust_type(reference),
            FieldKind::Map(MapField {
                ref key, ref value, ..
            }) => RustType::HashMap(
                Box::new(key.rust_storage_elem_type(reference)),
                Box::new(value.rust_storage_elem_type(reference)),
            ),
            FieldKind::Singular(ref singular) => singular.rust_storage_type(reference),
            FieldKind::Oneof(..) => unreachable!(),
        }
    }

    // type of `v` in `for v in field`
    fn full_storage_iter_elem_type(&self, reference: &FileAndMod) -> RustType {
        if let FieldKind::Oneof(ref oneof) = self.kind {
            oneof.elem.rust_storage_elem_type(reference)
        } else {
            self.full_storage_type(reference).iter_elem_type()
        }
    }

    // suffix `xxx` as in `os.write_xxx_no_tag(..)`
    fn os_write_fn_suffix(&self) -> &str {
        protobuf_name(self.proto_type)
    }

    fn os_write_fn_suffix_with_unknown_for_enum(&self) -> &str {
        if self.proto_type == field_descriptor_proto::Type::TYPE_ENUM {
            "enum_or_unknown"
        } else {
            self.os_write_fn_suffix()
        }
    }

    // type of `v` in `os.write_xxx_no_tag(v)`
    fn os_write_fn_param_type(&self) -> RustType {
        match self.proto_type {
            field_descriptor_proto::Type::TYPE_STRING => RustType::Ref(Box::new(RustType::Str)),
            field_descriptor_proto::Type::TYPE_BYTES => {
                RustType::Ref(Box::new(RustType::Slice(Box::new(RustType::Int(false, 8)))))
            }
            field_descriptor_proto::Type::TYPE_ENUM => RustType::Int(true, 32),
            t => rust_name(t),
        }
    }

    // for field `foo`, type of param of `fn set_foo(..)`
    fn set_xxx_param_type(&self, reference: &FileAndMod) -> RustType {
        match self.kind {
            FieldKind::Singular(SingularField { ref elem, .. })
            | FieldKind::Oneof(OneofField { ref elem, .. }) => {
                elem.rust_set_xxx_param_type(reference)
            }
            FieldKind::Repeated(..) | FieldKind::Map(..) => self.full_storage_type(reference),
        }
    }

    // for field `foo`, return type if `fn take_foo(..)`
    fn take_xxx_return_type(&self, reference: &FileAndMod) -> RustType {
        self.set_xxx_param_type(reference)
    }

    // for field `foo`, return type of `fn mut_foo(..)`
    fn mut_xxx_return_type(&self, reference: &FileAndMod) -> RustType {
        RustType::Ref(Box::new(match self.kind {
            FieldKind::Singular(SingularField { ref elem, .. })
            | FieldKind::Oneof(OneofField { ref elem, .. }) => {
                elem.rust_storage_elem_type(reference)
            }
            FieldKind::Repeated(..) | FieldKind::Map(..) => self.full_storage_type(reference),
        }))
    }

    // for field `foo`, return type of `fn foo(..)`
    fn getter_return_type(&self) -> RustType {
        let reference = self
            .proto_field
            .message
            .scope
            .file_and_mod(self.customize.clone());
        match &self.kind {
            FieldKind::Singular(s) => {
                SingularOrOneofField::Singular(s.clone()).getter_return_type(&reference)
            }
            FieldKind::Oneof(o) => {
                SingularOrOneofField::Oneof(o.clone()).getter_return_type(&reference)
            }
            FieldKind::Repeated(RepeatedField { ref elem, .. }) => RustType::Ref(Box::new(
                RustType::Slice(Box::new(elem.rust_storage_elem_type(&reference))),
            )),
            FieldKind::Map(..) => RustType::Ref(Box::new(self.full_storage_type(&reference))),
        }
    }

    // fixed size type?
    fn is_fixed(&self) -> bool {
        field_type_size(self.proto_type).is_some()
    }

    // must use zigzag encoding?
    fn is_zigzag(&self) -> bool {
        match self.proto_type {
            field_descriptor_proto::Type::TYPE_SINT32
            | field_descriptor_proto::Type::TYPE_SINT64 => true,
            _ => false,
        }
    }

    // data is enum
    fn is_enum(&self) -> bool {
        match self.proto_type {
            field_descriptor_proto::Type::TYPE_ENUM => true,
            _ => false,
        }
    }

    // elem data is not stored in heap
    pub fn elem_type_is_copy(&self) -> bool {
        type_is_copy(self.proto_type)
    }

    fn defaut_value_from_proto_float(f: f64, type_name: &str) -> String {
        if f.is_nan() {
            format!("::std::{}::NAN", type_name)
        } else if f.is_infinite() {
            if f > 0.0 {
                format!("::std::{}::INFINITY", type_name)
            } else {
                format!("::std::{}::NEG_INFINITY", type_name)
            }
        } else {
            format!("{:?}{}", f, type_name)
        }
    }

    fn singular_or_oneof_default_value_from_proto(&self, elem: &FieldElem) -> Option<String> {
        if !self.proto_field.field.proto().has_default_value() {
            return None;
        }

        let default_value = self.proto_field.field.singular_default_value();
        Some(match default_value {
            ReflectValueRef::Bool(b) => format!("{}", b),
            ReflectValueRef::I32(v) => format!("{}i32", v),
            ReflectValueRef::I64(v) => format!("{}i64", v),
            ReflectValueRef::U32(v) => format!("{}u32", v),
            ReflectValueRef::U64(v) => format!("{}u64", v),
            ReflectValueRef::String(v) => rust::quote_escape_str(v),
            ReflectValueRef::Bytes(v) => rust::quote_escape_bytes(v),
            ReflectValueRef::F32(v) => Self::defaut_value_from_proto_float(v as f64, "f32"),
            ReflectValueRef::F64(v) => Self::defaut_value_from_proto_float(v as f64, "f64"),
            ReflectValueRef::Enum(_e, _v) => {
                if let &FieldElem::Enum(ref e) = elem {
                    format!("{}", e.default_value_rust_expr(&self.file_and_mod()))
                } else {
                    unreachable!()
                }
            }
            t => panic!("default value is not implemented for type: {:?}", t),
        })
    }

    fn default_value_from_proto(&self) -> Option<String> {
        match self.kind {
            FieldKind::Oneof(OneofField { ref elem, .. })
            | FieldKind::Singular(SingularField { ref elem, .. }) => {
                self.singular_or_oneof_default_value_from_proto(elem)
            }
            _ => unreachable!(),
        }
    }

    fn default_value_from_proto_typed(&self) -> Option<RustValueTyped> {
        self.default_value_from_proto().map(|v| {
            let default_value_type = match self.proto_type {
                field_descriptor_proto::Type::TYPE_STRING => RustType::Ref(Box::new(RustType::Str)),
                field_descriptor_proto::Type::TYPE_BYTES => {
                    RustType::Ref(Box::new(RustType::Slice(Box::new(RustType::u8()))))
                }
                _ => self.full_storage_iter_elem_type(
                    &self
                        .proto_field
                        .message
                        .scope
                        .file_and_mod(self.customize.clone()),
                ),
            };

            RustValueTyped {
                value: v,
                rust_type: default_value_type,
            }
        })
    }

    // default value to be returned from `fn xxx` for field `xxx`.
    fn xxx_default_value_rust(&self) -> String {
        match self.kind {
            FieldKind::Singular(..) | FieldKind::Oneof(..) => {
                self.default_value_from_proto().unwrap_or_else(|| {
                    self.getter_return_type()
                        .default_value(&self.customize, false)
                })
            }
            _ => unreachable!(),
        }
    }

    // default to be assigned to field
    fn element_default_value_rust(&self) -> RustValueTyped {
        match self.kind {
            FieldKind::Singular(..) | FieldKind::Oneof(..) => {
                self.default_value_from_proto_typed().unwrap_or_else(|| {
                    self.elem()
                        .rust_storage_elem_type(
                            &self
                                .proto_field
                                .message
                                .scope
                                .file_and_mod(self.customize.clone()),
                        )
                        .default_value_typed(&self.customize, false)
                })
            }
            _ => unreachable!(),
        }
    }

    pub fn reconstruct_def(&self) -> String {
        let prefix = match (self.proto_field.field.proto().label(), self.syntax) {
            (field_descriptor_proto::Label::LABEL_REPEATED, _) => "repeated ",
            (_, Syntax::Proto3) => "",
            (field_descriptor_proto::Label::LABEL_OPTIONAL, _) => "optional ",
            (field_descriptor_proto::Label::LABEL_REQUIRED, _) => "required ",
        };
        format!(
            "{}{} {} = {}",
            prefix,
            field_type_protobuf_name(self.proto_field.field.proto()),
            self.proto_field.name(),
            self.proto_field.number()
        )
    }

    pub fn write_clear(&self, w: &mut CodeWriter) {
        match self.kind {
            FieldKind::Oneof(ref o) => {
                w.write_line(&format!(
                    "self.{} = ::std::option::Option::None;",
                    o.oneof_field_name
                ));
            }
            _ => {
                let clear_expr = self
                    .full_storage_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    )
                    .clear(&self.self_field(), &self.customize);
                w.write_line(&format!("{};", clear_expr));
            }
        }
    }

    // expression that returns size of data is variable
    fn element_size(&self, var: &str, var_type: &RustType) -> String {
        assert!(!self.is_repeated_packed());

        match field_type_size(self.proto_type) {
            Some(data_size) => format!("{}", data_size + self.tag_size()),
            None => match self.proto_type {
                field_descriptor_proto::Type::TYPE_MESSAGE => panic!("not a single-liner"),
                // We are not inlining `bytes_size` here,
                // assuming the compiler is smart enough to do it for us.
                // https://rust.godbolt.org/z/GrKa5zxq6
                field_descriptor_proto::Type::TYPE_BYTES => format!(
                    "{}::rt::bytes_size({}, &{})",
                    protobuf_crate_path(&self.customize),
                    self.proto_field.number(),
                    var
                ),
                field_descriptor_proto::Type::TYPE_STRING => format!(
                    "{}::rt::string_size({}, &{})",
                    protobuf_crate_path(&self.customize),
                    self.proto_field.number(),
                    var
                ),
                field_descriptor_proto::Type::TYPE_ENUM => {
                    let param_type = match var_type {
                        &RustType::Ref(ref t) => (**t).clone(),
                        t => t.clone(),
                    };
                    format!(
                        "{}::rt::enum_or_unknown_size({}, {})",
                        protobuf_crate_path(&self.customize),
                        self.proto_field.number(),
                        var_type.into_target(&param_type, var, &self.customize)
                    )
                }
                _ => {
                    let param_type = match var_type {
                        &RustType::Ref(ref t) => (**t).clone(),
                        t => t.clone(),
                    };
                    if self.proto_type.is_s_varint() {
                        format!(
                            "{}::rt::value_varint_zigzag_size({}, {})",
                            protobuf_crate_path(&self.customize),
                            self.proto_field.number(),
                            var_type.into_target(&param_type, var, &self.customize)
                        )
                    } else {
                        format!(
                            "{}::rt::value_size({}, {}, {}::rt::WireType::{:?})",
                            protobuf_crate_path(&self.customize),
                            self.proto_field.number(),
                            var_type.into_target(&param_type, var, &self.customize),
                            protobuf_crate_path(&self.customize),
                            self.wire_type
                        )
                    }
                }
            },
        }
    }

    // output code that writes single element to stream
    pub fn write_write_element(&self, w: &mut CodeWriter, os: &str, v: &RustValueTyped) {
        if let FieldKind::Repeated(RepeatedField { packed: true, .. }) = self.kind {
            unreachable!();
        };

        match self.proto_type {
            field_descriptor_proto::Type::TYPE_MESSAGE => {
                let param_type = RustType::Ref(Box::new(
                    self.elem().rust_storage_elem_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    ),
                ));

                w.write_line(&format!(
                    "{}::rt::write_message_field_with_cached_size({}, {}, {})?;",
                    protobuf_crate_path(&self.customize),
                    self.proto_field.number(),
                    v.into_type(param_type, &self.customize).value,
                    os
                ));
            }
            _ => {
                let param_type = self.os_write_fn_param_type();
                let os_write_fn_suffix = self.os_write_fn_suffix();
                let number = self.proto_field.number();
                w.write_line(&format!(
                    "{}.write_{}({}, {})?;",
                    os,
                    os_write_fn_suffix,
                    number,
                    v.into_type(param_type, &self.customize).value
                ));
            }
        }
    }

    fn self_field(&self) -> String {
        format!("self.{}", self.rust_name)
    }

    fn self_field_is_some(&self) -> String {
        assert!(self.is_singular());
        format!("{}.is_some()", self.self_field())
    }

    fn self_field_is_not_empty(&self) -> String {
        assert!(self.is_repeated_or_map());
        format!("!{}.is_empty()", self.self_field())
    }

    fn self_field_is_none(&self) -> String {
        assert!(self.is_singular());
        format!("{}.is_none()", self.self_field())
    }

    // field data viewed as Option
    fn self_field_as_option(&self, elem: &FieldElem, option_kind: OptionKind) -> RustValueTyped {
        match self.full_storage_type(
            &self
                .proto_field
                .message
                .scope
                .file_and_mod(self.customize.clone()),
        ) {
            RustType::Option(ref e) if e.is_copy() => {
                return RustType::Option(e.clone()).value(self.self_field());
            }
            _ => {}
        };

        //        let as_option_type = RustType::Option(Box::new(elem.rust_storage_elem_type().ref_type()));
        //
        //        // TODO: do not use as_option_ref, return Box for OptionBox instead (for simpler code)
        //        let v = option_kind.as_option_ref(&self.self_field());
        //
        //        as_option_type.value(v)

        let as_option_type = option_kind.as_ref_type(
            elem.rust_storage_elem_type(
                &self
                    .proto_field
                    .message
                    .scope
                    .file_and_mod(self.customize.clone()),
            ),
        );

        as_option_type.value(format!("{}.as_ref()", self.self_field()))
    }

    /// Field visibility in message struct
    fn visibility(&self) -> Visibility {
        if self.expose_field {
            Visibility::Public
        } else {
            match self.kind {
                FieldKind::Repeated(..) => Visibility::Default,
                FieldKind::Singular(SingularField { ref flag, .. }) => match *flag {
                    SingularFieldFlag::WithFlag { .. } => Visibility::Default,
                    SingularFieldFlag::WithoutFlag => Visibility::Public,
                },
                FieldKind::Map(..) => Visibility::Public,
                FieldKind::Oneof(..) => unreachable!(),
            }
        }
    }

    pub fn write_struct_field(&self, w: &mut CodeWriter) {
        if self.proto_type == field_descriptor_proto::Type::TYPE_GROUP {
            w.comment(&format!("{}: <group>", &self.rust_name));
        } else {
            w.all_documentation(self.info, &self.path);

            write_protoc_insertion_point_for_field(w, &self.customize, &self.proto_field.field);
            let vis = self.visibility();
            w.field_decl_vis(
                vis,
                self.rust_name.get(),
                &self
                    .full_storage_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    )
                    .to_code(&self.customize),
            );
        }
    }

    fn write_if_let_self_field_is_some<F>(&self, s: &SingularField, w: &mut CodeWriter, cb: F)
    where
        F: Fn(&RustValueTyped, &mut CodeWriter),
    {
        match s {
            SingularField {
                flag: SingularFieldFlag::WithFlag { option_kind, .. },
                ref elem,
            } => {
                let var = "v";
                let ref_prefix = match elem
                    .rust_storage_elem_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    )
                    .is_copy()
                {
                    true => "",
                    false => "",
                };
                let as_option = self.self_field_as_option(elem, *option_kind);
                w.if_let_stmt(
                    &format!("Some({}{})", ref_prefix, var),
                    &as_option.value,
                    |w| {
                        let v = RustValueTyped {
                            value: var.to_owned(),
                            rust_type: as_option.rust_type.elem_type(),
                        };
                        cb(&v, w);
                    },
                );
            }
            SingularField {
                flag: SingularFieldFlag::WithoutFlag,
                ref elem,
            } => match *elem {
                FieldElem::Primitive(field_descriptor_proto::Type::TYPE_STRING, ..)
                | FieldElem::Primitive(field_descriptor_proto::Type::TYPE_BYTES, ..) => {
                    w.if_stmt(format!("!{}.is_empty()", self.self_field()), |w| {
                        let v = RustValueTyped {
                            value: self.self_field(),
                            rust_type: self.full_storage_type(
                                &self
                                    .proto_field
                                    .message
                                    .scope
                                    .file_and_mod(self.customize.clone()),
                            ),
                        };
                        cb(&v, w);
                    });
                }
                _ => {
                    w.if_stmt(
                        format!(
                            "{} != {}",
                            self.self_field(),
                            self.full_storage_type(
                                &self
                                    .proto_field
                                    .message
                                    .scope
                                    .file_and_mod(self.customize.clone())
                            )
                            .default_value(&self.customize, false)
                        ),
                        |w| {
                            let v = RustValueTyped {
                                value: self.self_field(),
                                rust_type: self.full_storage_type(
                                    &self
                                        .proto_field
                                        .message
                                        .scope
                                        .file_and_mod(self.customize.clone()),
                                ),
                            };
                            cb(&v, w);
                        },
                    );
                }
            },
        }
    }

    fn write_if_self_field_is_not_empty<F>(&self, w: &mut CodeWriter, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        assert!(self.is_repeated_or_map());
        let self_field_is_not_empty = self.self_field_is_not_empty();
        w.if_stmt(self_field_is_not_empty, cb);
    }

    pub fn write_if_self_field_is_none<F>(&self, w: &mut CodeWriter, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        let self_field_is_none = self.self_field_is_none();
        w.if_stmt(self_field_is_none, cb)
    }

    // repeated or singular
    pub fn write_for_self_field<F>(&self, w: &mut CodeWriter, varn: &str, cb: F)
    where
        F: Fn(&mut CodeWriter, &RustType),
    {
        let file_and_mod = self
            .proto_field
            .message
            .scope
            .file_and_mod(self.customize.clone());

        match &self.kind {
            FieldKind::Oneof(oneof_field) => {
                let cond = format!(
                    "Some({}(ref {}))",
                    oneof_field.variant_path(&file_and_mod.relative_mod.clone().into_path()),
                    varn
                );
                w.if_let_stmt(
                    &cond,
                    &format!("self.{}", oneof_field.oneof_field_name),
                    |w| cb(w, &oneof_field.elem.rust_storage_elem_type(&file_and_mod)),
                )
            }
            _ => {
                let v_type = self.full_storage_iter_elem_type(&file_and_mod);
                let self_field = self.self_field();
                w.for_stmt(&format!("&{}", self_field), varn, |w| cb(w, &v_type));
            }
        }
    }

    fn write_self_field_assign(&self, w: &mut CodeWriter, value: &str) {
        let self_field = self.self_field();
        w.write_line(&format!("{} = {};", self_field, value));
    }

    fn write_self_field_assign_some(&self, w: &mut CodeWriter, s: &SingularField, value: &str) {
        match s {
            &SingularField {
                flag: SingularFieldFlag::WithFlag { option_kind, .. },
                ..
            } => {
                self.write_self_field_assign(w, &option_kind.wrap_value(value, &self.customize));
            }
            &SingularField {
                flag: SingularFieldFlag::WithoutFlag,
                ..
            } => {
                self.write_self_field_assign(w, value);
            }
        }
    }

    fn write_self_field_assign_value_singular(
        &self,
        w: &mut CodeWriter,
        s: &SingularField,
        value: &RustValueTyped,
    ) {
        let SingularField { ref elem, ref flag } = s;
        let converted = value.into_type(
            elem.rust_storage_elem_type(
                &self
                    .proto_field
                    .message
                    .scope
                    .file_and_mod(self.customize.clone()),
            )
            .clone(),
            &self.customize,
        );
        let wrapped = match flag {
            SingularFieldFlag::WithoutFlag => converted.value,
            SingularFieldFlag::WithFlag { option_kind, .. } => {
                option_kind.wrap_value(&converted.value, &self.customize)
            }
        };
        self.write_self_field_assign(w, &wrapped);
    }

    fn write_self_field_assign_value(&self, w: &mut CodeWriter, value: &RustValueTyped) {
        match self.kind {
            FieldKind::Repeated(..) | FieldKind::Map(..) => {
                let converted = value.into_type(
                    self.full_storage_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    ),
                    &self.customize,
                );
                self.write_self_field_assign(w, &converted.value);
            }
            FieldKind::Singular(ref s) => {
                self.write_self_field_assign_value_singular(w, s, value);
            }
            FieldKind::Oneof(..) => unreachable!(),
        }
    }

    fn write_self_field_assign_default(
        &self,
        field_kind: &SingularOrOneofField,
        w: &mut CodeWriter,
    ) {
        match field_kind {
            SingularOrOneofField::Oneof(oneof) => {
                w.write_line(format!(
                    "self.{} = ::std::option::Option::Some({}({}))",
                    oneof.oneof_field_name,
                    oneof.variant_path(
                        &self
                            .proto_field
                            .message
                            .scope
                            .rust_path_to_file()
                            .clone()
                            .into_path()
                    ),
                    // TODO: default from .proto is not needed here (?)
                    self.element_default_value_rust()
                        .into_type(
                            self.full_storage_iter_elem_type(
                                &self
                                    .proto_field
                                    .message
                                    .scope
                                    .file_and_mod(self.customize.clone())
                            ),
                            &self.customize
                        )
                        .value
                ));
            }
            SingularOrOneofField::Singular(singular) => {
                // Note it is different from C++ protobuf, where field is initialized
                // with default value
                match singular.flag {
                    SingularFieldFlag::WithFlag { option_kind, .. } => match option_kind {
                        OptionKind::MessageField => {
                            let self_field = self.self_field();
                            w.write_line(&format!("{}.set_default();", self_field));
                        }
                        _ => {
                            self.write_self_field_assign_some(
                                w,
                                singular,
                                &self
                                    .elem()
                                    .rust_storage_elem_type(
                                        &self
                                            .proto_field
                                            .message
                                            .scope
                                            .file_and_mod(self.customize.clone()),
                                    )
                                    .default_value_typed(&self.customize, false)
                                    .into_type(
                                        singular.elem.rust_storage_elem_type(
                                            &self
                                                .proto_field
                                                .message
                                                .scope
                                                .file_and_mod(self.customize.clone()),
                                        ),
                                        &self.customize,
                                    )
                                    .value,
                            );
                        }
                    },
                    SingularFieldFlag::WithoutFlag => unimplemented!(),
                }
            }
        }
    }

    fn self_field_vec_packed_fixed_size(&self) -> String {
        format!(
            "{}::rt::vec_packed_fixed_size({}, &{})",
            protobuf_crate_path(&self.customize),
            self.proto_field.number(),
            self.self_field()
        )
    }

    fn self_field_vec_packed_varint_size(&self) -> String {
        // zero is filtered outside
        assert!(!self.is_fixed());
        let fn_name = if self.is_enum() {
            "vec_packed_enum_or_unknown_size"
        } else {
            if self.is_zigzag() {
                "vec_packed_varint_zigzag_size"
            } else {
                "vec_packed_varint_size"
            }
        };
        format!(
            "{}::rt::{}({}, &{})",
            protobuf_crate_path(&self.customize),
            fn_name,
            self.proto_field.number(),
            self.self_field()
        )
    }

    pub fn clear_field_func(&self) -> String {
        format!("clear_{}", self.rust_name)
    }

    fn write_merge_from_field_message_string_bytes_repeated(
        &self,
        r: &RepeatedField,
        w: &mut CodeWriter,
    ) {
        let read_fn = match &r.elem {
            FieldElem::Message(..) => "read_message",
            FieldElem::Primitive(Type::TYPE_STRING, PrimitiveTypeVariant::Default) => "read_string",
            FieldElem::Primitive(Type::TYPE_STRING, PrimitiveTypeVariant::TokioBytes) => {
                "read_tokio_chars"
            }
            FieldElem::Primitive(Type::TYPE_BYTES, PrimitiveTypeVariant::Default) => "read_bytes",
            FieldElem::Primitive(Type::TYPE_BYTES, PrimitiveTypeVariant::TokioBytes) => {
                "read_tokio_bytes"
            }
            _ => unreachable!("for field {}", self.proto_field.field),
        };
        w.write_line(&format!("self.{}.push(is.{}()?);", self.rust_name, read_fn,));
    }

    fn tag_with_wire_type(&self, wire_type: WireType) -> u32 {
        (self.proto_field.number() << 3) + (wire_type as u32)
    }

    fn tag(&self) -> u32 {
        self.tag_with_wire_type(self.wire_type)
    }

    // Write `merge_from` part for this oneof field
    fn write_merge_from_oneof_case_block(&self, o: &OneofField, w: &mut CodeWriter) {
        w.case_block(&format!("{}", self.tag()), |w| {
            let typed = RustValueTyped {
                value: format!(
                    "{}?",
                    self.proto_type.read("is", o.elem.primitive_type_variant())
                ),
                rust_type: self.full_storage_iter_elem_type(
                    &self
                        .proto_field
                        .message
                        .scope
                        .file_and_mod(self.customize.clone()),
                ),
            };

            let maybe_boxed = if o.boxed {
                typed.boxed(&self.customize)
            } else {
                typed
            };

            w.write_line(&format!(
                "self.{} = ::std::option::Option::Some({}({}));",
                o.oneof_field_name,
                o.variant_path(
                    &self
                        .proto_field
                        .message
                        .scope
                        .rust_path_to_file()
                        .clone()
                        .into_path()
                ),
                maybe_boxed.value
            )); // TODO: into_type
        })
    }

    // Write `merge_from` part for this map field
    fn write_merge_from_map_case_block(&self, w: &mut CodeWriter) {
        let &MapField {
            ref key, ref value, ..
        } = self.map();
        w.case_block(&format!("{}", self.tag()), |w| {
            w.write_line(&format!(
                "{}::rt::read_map_into::<{}, {}>(is, &mut {})?;",
                protobuf_crate_path(&self.customize),
                key.lib_protobuf_type(&self.file_and_mod()),
                value.lib_protobuf_type(&self.file_and_mod()),
                self.self_field()
            ));
        })
    }

    // Write `merge_from` part for this singular field
    fn write_merge_from_singular_case_block(&self, s: &SingularField, w: &mut CodeWriter) {
        w.case_block(&format!("{}", self.tag()), |w| match s.elem {
            FieldElem::Message(..) => {
                w.write_line(&format!(
                    "{}::rt::read_singular_message_into_field(is, &mut self.{})?;",
                    protobuf_crate_path(&self.customize),
                    self.rust_name,
                ));
            }
            _ => {
                let read_proc = format!(
                    "{}?",
                    self.proto_type.read("is", s.elem.primitive_type_variant())
                );
                self.write_self_field_assign_some(w, s, &read_proc);
            }
        })
    }

    // Write `merge_from` part for this repeated field
    fn write_merge_from_repeated_case_block(&self, w: &mut CodeWriter) {
        let field = match self.kind {
            FieldKind::Repeated(ref field) => field,
            _ => panic!(),
        };

        match field.elem {
            FieldElem::Message(..)
            | FieldElem::Primitive(field_descriptor_proto::Type::TYPE_STRING, ..)
            | FieldElem::Primitive(field_descriptor_proto::Type::TYPE_BYTES, ..) => {
                w.case_block(&format!("{}", self.tag()), |w| {
                    self.write_merge_from_field_message_string_bytes_repeated(field, w);
                })
            }
            FieldElem::Enum(..) => {
                w.case_block(
                    &format!("{}", self.tag_with_wire_type(WireType::Varint)),
                    |w| {
                        w.write_line(&format!(
                            "self.{}.push(is.read_enum_or_unknown()?);",
                            self.rust_name,
                        ));
                    },
                );
                w.case_block(
                    &format!("{}", self.tag_with_wire_type(WireType::LengthDelimited)),
                    |w| {
                        w.write_line(&format!(
                            "{}::rt::read_repeated_packed_enum_or_unknown_into(is, &mut self.{})?",
                            protobuf_crate_path(&self.customize),
                            self.rust_name,
                        ));
                    },
                );
            }
            _ => {
                assert_ne!(self.wire_type, WireType::LengthDelimited);
                w.case_block(
                    &format!("{}", self.tag_with_wire_type(WireType::LengthDelimited)),
                    |w| {
                        w.write_line(&format!(
                            "is.read_repeated_packed_{}_into(&mut self.{})?;",
                            protobuf_name(self.proto_type),
                            self.rust_name
                        ));
                    },
                );
                w.case_block(&format!("{}", self.tag()), |w| {
                    w.write_line(&format!(
                        "self.{}.push(is.read_{}()?);",
                        self.rust_name,
                        protobuf_name(self.proto_type),
                    ));
                });
            }
        }
    }

    /// Write `merge_from` part for this field
    pub(crate) fn write_merge_from_field_case_block(&self, w: &mut CodeWriter) {
        match self.kind {
            FieldKind::Oneof(ref f) => self.write_merge_from_oneof_case_block(&f, w),
            FieldKind::Map(..) => self.write_merge_from_map_case_block(w),
            FieldKind::Singular(ref s) => self.write_merge_from_singular_case_block(s, w),
            FieldKind::Repeated(..) => self.write_merge_from_repeated_case_block(w),
        }
    }

    fn self_field_vec_packed_size(&self) -> String {
        match self.kind {
            FieldKind::Repeated(RepeatedField { packed: true, .. }) => {
                // zero is filtered outside
                if self.is_fixed() {
                    self.self_field_vec_packed_fixed_size()
                } else {
                    self.self_field_vec_packed_varint_size()
                }
            }
            _ => {
                panic!("not packed");
            }
        }
    }

    pub fn write_element_size(
        &self,
        w: &mut CodeWriter,
        item_var: &str,
        item_var_type: &RustType,
        sum_var: &str,
    ) {
        assert!(!self.is_repeated_packed());

        match self.proto_type {
            field_descriptor_proto::Type::TYPE_MESSAGE => {
                w.write_line(&format!("let len = {}.compute_size();", item_var));
                let tag_size = self.tag_size();
                w.write_line(&format!(
                    "{} += {} + {}::rt::compute_raw_varint64_size(len) + len;",
                    sum_var,
                    tag_size,
                    protobuf_crate_path(&self.customize),
                ));
            }
            _ => {
                w.write_line(&format!(
                    "{} += {};",
                    sum_var,
                    self.element_size(item_var, item_var_type)
                ));
            }
        }
    }

    pub fn write_message_write_field(&self, w: &mut CodeWriter) {
        match self.kind {
            FieldKind::Singular(ref s) => {
                self.write_if_let_self_field_is_some(s, w, |v, w| {
                    self.write_write_element(w, "os", &v);
                });
            }
            FieldKind::Repeated(RepeatedField { packed: false, .. }) => {
                self.write_for_self_field(w, "v", |w, v_type| {
                    let v = RustValueTyped {
                        value: "v".to_owned(),
                        rust_type: v_type.clone(),
                    };
                    self.write_write_element(w, "os", &v);
                });
            }
            FieldKind::Repeated(RepeatedField { packed: true, .. }) => {
                w.write_line(&format!(
                    "os.write_repeated_packed_{}({}, &{})?;",
                    self.os_write_fn_suffix_with_unknown_for_enum(),
                    self.proto_field.number(),
                    self.self_field()
                ));
            }
            FieldKind::Map(MapField {
                ref key, ref value, ..
            }) => {
                w.write_line(&format!(
                    "{}::rt::write_map_with_cached_sizes::<{}, {}>({}, &{}, os)?;",
                    protobuf_crate_path(&self.customize),
                    key.lib_protobuf_type(&self.file_and_mod()),
                    value.lib_protobuf_type(&self.file_and_mod()),
                    self.proto_field.number(),
                    self.self_field()
                ));
            }
            FieldKind::Oneof(..) => unreachable!(),
        };
    }

    pub fn write_message_compute_field_size(&self, sum_var: &str, w: &mut CodeWriter) {
        match self.kind {
            FieldKind::Singular(ref s) => {
                self.write_if_let_self_field_is_some(s, w, |v, w| {
                    match field_type_size(self.proto_type) {
                        Some(s) => {
                            let tag_size = self.tag_size();
                            w.write_line(&format!("{} += {};", sum_var, (s + tag_size) as isize));
                        }
                        None => {
                            self.write_element_size(w, &v.value, &v.rust_type, sum_var);
                        }
                    };
                });
            }
            FieldKind::Repeated(RepeatedField { packed: false, .. }) => {
                match field_type_size(self.proto_type) {
                    Some(s) => {
                        let tag_size = self.tag_size();
                        let self_field = self.self_field();
                        w.write_line(&format!(
                            "{} += {} * {}.len() as u64;",
                            sum_var,
                            (s + tag_size) as isize,
                            self_field
                        ));
                    }
                    None => {
                        self.write_for_self_field(w, "value", |w, value_type| {
                            self.write_element_size(w, "value", value_type, sum_var);
                        });
                    }
                };
            }
            FieldKind::Repeated(RepeatedField { packed: true, .. }) => {
                self.write_if_self_field_is_not_empty(w, |w| {
                    let size_expr = self.self_field_vec_packed_size();
                    w.write_line(&format!("{} += {};", sum_var, size_expr));
                });
            }
            FieldKind::Map(MapField {
                ref key, ref value, ..
            }) => {
                w.write_line(&format!(
                    "{} += {}::rt::compute_map_size::<{}, {}>({}, &{});",
                    sum_var,
                    protobuf_crate_path(&self.customize),
                    key.lib_protobuf_type(&self.file_and_mod()),
                    value.lib_protobuf_type(&self.file_and_mod()),
                    self.proto_field.number(),
                    self.self_field()
                ));
            }
            FieldKind::Oneof(..) => unreachable!(),
        }
    }

    fn write_message_field_get_singular_message(&self, s: &SingularField, w: &mut CodeWriter) {
        match s.flag {
            SingularFieldFlag::WithoutFlag => unimplemented!(),
            SingularFieldFlag::WithFlag { option_kind, .. } => {
                let self_field = self.self_field();
                let ref field_type_name = self.elem().rust_storage_elem_type(
                    &self
                        .proto_field
                        .message
                        .scope
                        .file_and_mod(self.customize.clone()),
                );
                w.write_line(option_kind.unwrap_ref_or_else(
                    &format!("{}.as_ref()", self_field),
                    &format!(
                        "<{} as {}::Message>::default_instance()",
                        field_type_name.to_code(&self.customize),
                        protobuf_crate_path(&self.customize),
                    ),
                ));
            }
        }
    }

    fn write_message_field_get_singular_enum(
        &self,
        flag: SingularFieldFlag,
        _elem: &FieldElemEnum,
        w: &mut CodeWriter,
    ) {
        match flag {
            SingularFieldFlag::WithoutFlag => {
                w.write_line(&format!("self.{}.enum_value_or_default()", self.rust_name));
            }
            SingularFieldFlag::WithFlag { .. } => {
                w.match_expr(&self.self_field(), |w| {
                    let default_value = self.xxx_default_value_rust();
                    w.case_expr("Some(e)", &format!("e.enum_value_or({})", default_value));
                    w.case_expr("None", &format!("{}", default_value));
                });
            }
        }
    }

    fn write_message_field_get_singular(&self, singular: &SingularField, w: &mut CodeWriter) {
        let get_xxx_return_type = self.getter_return_type();

        match singular.elem {
            FieldElem::Message(..) => self.write_message_field_get_singular_message(singular, w),
            FieldElem::Enum(ref en) => {
                self.write_message_field_get_singular_enum(singular.flag, en, w)
            }
            _ => {
                let get_xxx_default_value_rust = self.xxx_default_value_rust();
                let self_field = self.self_field();
                match singular {
                    &SingularField {
                        ref elem,
                        flag: SingularFieldFlag::WithFlag { option_kind, .. },
                        ..
                    } => {
                        if get_xxx_return_type.is_ref().is_some() {
                            let as_option = self.self_field_as_option(elem, option_kind);
                            w.match_expr(&as_option.value, |w| {
                                let v_type = as_option.rust_type.elem_type();
                                let r_type = self.getter_return_type();
                                w.case_expr(
                                    "Some(v)",
                                    v_type.into_target(&r_type, "v", &self.customize),
                                );
                                let get_xxx_default_value_rust = self.xxx_default_value_rust();
                                w.case_expr("None", get_xxx_default_value_rust);
                            });
                        } else {
                            w.write_line(&format!(
                                "{}.unwrap_or({})",
                                self_field, get_xxx_default_value_rust
                            ));
                        }
                    }
                    &SingularField {
                        flag: SingularFieldFlag::WithoutFlag,
                        ..
                    } => {
                        w.write_line(
                            self.full_storage_type(
                                &self
                                    .proto_field
                                    .message
                                    .scope
                                    .file_and_mod(self.customize.clone()),
                            )
                            .into_target(
                                &get_xxx_return_type,
                                &self_field,
                                &self.customize,
                            ),
                        );
                    }
                }
            }
        }
    }

    fn write_message_field_get_oneof(&self, o: &OneofField, w: &mut CodeWriter) {
        let get_xxx_return_type = SingularOrOneofField::Oneof(o.clone()).getter_return_type(
            &self
                .proto_field
                .message
                .scope
                .file_and_mod(self.customize.clone()),
        );
        let OneofField { ref elem, .. } = o;
        w.match_expr(&format!("self.{}", o.oneof_field_name), |w| {
            let (refv, vtype) = if !elem.is_copy() {
                (
                    "ref v",
                    elem.rust_storage_elem_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    )
                    .ref_type(),
                )
            } else {
                (
                    "v",
                    elem.rust_storage_elem_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    ),
                )
            };
            w.case_expr(
                format!(
                    "::std::option::Option::Some({}({}))",
                    o.variant_path(
                        &self
                            .proto_field
                            .message
                            .scope
                            .rust_path_to_file()
                            .clone()
                            .into_path()
                    ),
                    refv
                ),
                vtype.into_target(&get_xxx_return_type, "v", &self.customize),
            );
            w.case_expr("_", self.xxx_default_value_rust());
        });
    }

    fn write_message_field_get(&self, w: &mut CodeWriter) {
        let get_xxx_return_type = self.getter_return_type();
        let fn_def = format!(
            "{}(&self) -> {}",
            self.rust_name,
            get_xxx_return_type.to_code(&self.customize)
        );

        w.pub_fn(&fn_def, |w| match self.kind {
            FieldKind::Oneof(ref o) => {
                self.write_message_field_get_oneof(o, w);
            }
            FieldKind::Singular(ref s) => {
                self.write_message_field_get_singular(s, w);
            }
            FieldKind::Repeated(..) | FieldKind::Map(..) => {
                let self_field = self.self_field();
                w.write_line(&format!("&{}", self_field));
            }
        });
    }

    fn has_has(&self) -> bool {
        match self.kind {
            FieldKind::Repeated(..) | FieldKind::Map(..) => false,
            FieldKind::Singular(SingularField {
                flag: SingularFieldFlag::WithFlag { .. },
                ..
            }) => true,
            FieldKind::Singular(SingularField {
                flag: SingularFieldFlag::WithoutFlag,
                ..
            }) => false,
            FieldKind::Oneof(..) => true,
        }
    }

    fn has_mut(&self) -> bool {
        match self.kind {
            FieldKind::Repeated(..) | FieldKind::Map(..) => true,
            // TODO: string should be public, and mut is not needed
            FieldKind::Singular(..) | FieldKind::Oneof(..) => !self.elem_type_is_copy(),
        }
    }

    fn has_take(&self) -> bool {
        match self.kind {
            FieldKind::Repeated(..) | FieldKind::Map(..) => true,
            // TODO: string should be public, and mut is not needed
            FieldKind::Singular(..) | FieldKind::Oneof(..) => !self.elem_type_is_copy(),
        }
    }

    fn has_name(&self) -> String {
        format!("has_{}", self.rust_name)
    }

    fn write_message_field_has(&self, w: &mut CodeWriter) {
        w.pub_fn(
            &format!("{}(&self) -> bool", self.has_name()),
            |w| match self.kind {
                FieldKind::Oneof(ref oneof) => {
                    w.match_expr(&format!("self.{}", oneof.oneof_field_name), |w| {
                        w.case_expr(
                            format!(
                                "::std::option::Option::Some({}(..))",
                                oneof.variant_path(
                                    &self
                                        .proto_field
                                        .message
                                        .scope
                                        .rust_path_to_file()
                                        .clone()
                                        .into_path()
                                )
                            ),
                            "true",
                        );
                        w.case_expr("_", "false");
                    });
                }
                _ => {
                    let self_field_is_some = self.self_field_is_some();
                    w.write_line(self_field_is_some);
                }
            },
        );
    }

    fn write_message_field_set(&self, w: &mut CodeWriter) {
        let set_xxx_param_type = self.set_xxx_param_type(
            &self
                .proto_field
                .message
                .scope
                .file_and_mod(self.customize.clone()),
        );
        w.comment("Param is passed by value, moved");
        let ref name = self.rust_name;
        w.pub_fn(
            &format!(
                "set_{}(&mut self, v: {})",
                name,
                set_xxx_param_type.to_code(&self.customize)
            ),
            |w| {
                let value_typed = RustValueTyped {
                    value: "v".to_owned(),
                    rust_type: set_xxx_param_type.clone(),
                };
                match self.kind {
                    FieldKind::Oneof(ref oneof) => {
                        let v = set_xxx_param_type.into_target(
                            &oneof.rust_type(
                                &self
                                    .proto_field
                                    .message
                                    .scope
                                    .file_and_mod(self.customize.clone()),
                            ),
                            "v",
                            &self.customize,
                        );
                        w.write_line(&format!(
                            "self.{} = ::std::option::Option::Some({}({}))",
                            oneof.oneof_field_name,
                            oneof.variant_path(
                                &self
                                    .proto_field
                                    .message
                                    .scope
                                    .rust_path_to_file()
                                    .clone()
                                    .into_path()
                            ),
                            v
                        ));
                    }
                    _ => {
                        self.write_self_field_assign_value(w, &value_typed);
                    }
                }
            },
        );
    }

    fn write_message_field_mut_singular(&self, s: &SingularField, w: &mut CodeWriter) {
        match s {
            SingularField {
                flag: SingularFieldFlag::WithFlag { .. },
                ..
            } => {
                self.write_if_self_field_is_none(w, |w| {
                    self.write_self_field_assign_default(
                        &SingularOrOneofField::Singular(s.clone()),
                        w,
                    );
                });
                let self_field = self.self_field();
                w.write_line(&format!("{}.as_mut().unwrap()", self_field));
            }
            SingularField {
                flag: SingularFieldFlag::WithoutFlag,
                ..
            } => w.write_line(&format!("&mut {}", self.self_field())),
        }
    }

    fn write_message_field_mut(&self, w: &mut CodeWriter) {
        let mut_xxx_return_type = self.mut_xxx_return_type(
            &self
                .proto_field
                .message
                .scope
                .file_and_mod(self.customize.clone()),
        );
        w.comment("Mutable pointer to the field.");
        if self.is_singular() {
            w.comment("If field is not initialized, it is initialized with default value first.");
        }
        let fn_def = match mut_xxx_return_type {
            RustType::Ref(ref param) => format!(
                "mut_{}(&mut self) -> &mut {}",
                self.rust_name,
                param.to_code(&self.customize)
            ),
            _ => panic!(
                "not a ref: {}",
                mut_xxx_return_type.to_code(&self.customize)
            ),
        };
        w.pub_fn(&fn_def, |w| {
            match self.kind {
                FieldKind::Repeated(..) | FieldKind::Map(..) => {
                    let self_field = self.self_field();
                    w.write_line(&format!("&mut {}", self_field));
                }
                FieldKind::Singular(ref s) => {
                    self.write_message_field_mut_singular(s, w);
                }
                FieldKind::Oneof(ref o) => {
                    let self_field_oneof = format!("self.{}", o.oneof_field_name);

                    // if oneof does not contain current field
                    w.if_let_else_stmt(
                        &format!(
                            "::std::option::Option::Some({}(_))",
                            o.variant_path(
                                &self
                                    .proto_field
                                    .message
                                    .scope
                                    .rust_path_to_file()
                                    .clone()
                                    .into_path()
                            )
                        )[..],
                        &self_field_oneof[..],
                        |w| {
                            // initialize it with default value
                            w.write_line(&format!(
                                "{} = ::std::option::Option::Some({}({}));",
                                self_field_oneof,
                                o.variant_path(
                                    &self
                                        .proto_field
                                        .message
                                        .scope
                                        .rust_path_to_file()
                                        .clone()
                                        .into_path()
                                ),
                                self.element_default_value_rust()
                                    .into_type(
                                        o.rust_type(
                                            &self
                                                .proto_field
                                                .message
                                                .scope
                                                .file_and_mod(self.customize.clone())
                                        ),
                                        &self.customize
                                    )
                                    .value
                            ));
                        },
                    );

                    // extract field
                    w.match_expr(self_field_oneof, |w| {
                        w.case_expr(
                            format!(
                                "::std::option::Option::Some({}(ref mut v))",
                                o.variant_path(
                                    &self
                                        .proto_field
                                        .message
                                        .scope
                                        .rust_path_to_file()
                                        .clone()
                                        .into_path()
                                )
                            ),
                            "v",
                        );
                        w.case_expr("_", "panic!()");
                    });
                }
            }
        });
    }

    fn write_message_field_take_oneof(&self, o: &OneofField, w: &mut CodeWriter) {
        let take_xxx_return_type = self.take_xxx_return_type(
            &self
                .proto_field
                .message
                .scope
                .file_and_mod(self.customize.clone()),
        );

        // TODO: replace with if let
        w.write_line(&format!("if self.{}() {{", self.has_name()));
        w.indented(|w| {
            let self_field_oneof = format!("self.{}", o.oneof_field_name);
            w.match_expr(format!("{}.take()", self_field_oneof), |w| {
                let value_in_some = o
                    .rust_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    )
                    .value("v".to_owned());
                let converted = value_in_some.into_type(
                    self.take_xxx_return_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    ),
                    &self.customize,
                );
                w.case_expr(
                    format!(
                        "::std::option::Option::Some({}(v))",
                        o.variant_path(
                            &self
                                .proto_field
                                .message
                                .scope
                                .rust_path_to_file()
                                .clone()
                                .into_path()
                        )
                    ),
                    &converted.value,
                );
                w.case_expr("_", "panic!()");
            });
        });
        w.write_line("} else {");
        w.indented(|w| {
            w.write_line(
                self.elem()
                    .rust_storage_elem_type(
                        &self
                            .proto_field
                            .message
                            .scope
                            .file_and_mod(self.customize.clone()),
                    )
                    .default_value_typed(&self.customize, false)
                    .into_type(take_xxx_return_type.clone(), &self.customize)
                    .value,
            );
        });
        w.write_line("}");
    }

    fn write_message_field_take_singular(&self, s: &SingularField, w: &mut CodeWriter) {
        match s {
            SingularField {
                ref elem,
                flag: SingularFieldFlag::WithFlag { option_kind, .. },
            } => {
                if !elem.is_copy() {
                    w.write_line(
                        &option_kind.unwrap_or_else(
                            &format!("{}.take()", self.self_field()),
                            &elem
                                .rust_storage_elem_type(
                                    &self
                                        .proto_field
                                        .message
                                        .scope
                                        .file_and_mod(self.customize.clone()),
                                )
                                .default_value(&self.customize, false),
                        ),
                    );
                } else {
                    w.write_line(&format!(
                        "{}.take().unwrap_or({})",
                        self.self_field(),
                        self.element_default_value_rust().value
                    ));
                }
            }
            SingularField {
                flag: SingularFieldFlag::WithoutFlag,
                ..
            } => w.write_line(&format!(
                "::std::mem::replace(&mut {}, {})",
                self.self_field(),
                self.full_storage_type(
                    &self
                        .proto_field
                        .message
                        .scope
                        .file_and_mod(self.customize.clone())
                )
                .default_value(&self.customize, false)
            )),
        }
    }

    fn write_message_field_take(&self, w: &mut CodeWriter) {
        let take_xxx_return_type = self.take_xxx_return_type(
            &self
                .proto_field
                .message
                .scope
                .file_and_mod(self.customize.clone()),
        );
        w.comment("Take field");
        w.pub_fn(
            &format!(
                "take_{}(&mut self) -> {}",
                self.rust_name,
                take_xxx_return_type.to_code(&self.customize)
            ),
            |w| match self.kind {
                FieldKind::Singular(ref s) => self.write_message_field_take_singular(&s, w),
                FieldKind::Oneof(ref o) => self.write_message_field_take_oneof(o, w),
                FieldKind::Repeated(..) | FieldKind::Map(..) => {
                    w.write_line(&format!(
                        "::std::mem::replace(&mut self.{}, {})",
                        self.rust_name,
                        take_xxx_return_type.default_value(&self.customize, false)
                    ));
                }
            },
        );
    }

    pub fn write_message_single_field_accessors(&self, w: &mut CodeWriter) {
        if self.generate_accessors || self.generate_getter {
            w.write_line("");
            let reconstruct_def = self.reconstruct_def();
            w.comment(&(reconstruct_def + ";"));
        }

        if self.generate_getter {
            w.write_line("");
            self.write_message_field_get(w);
        }

        if !self.generate_accessors {
            return;
        }

        w.write_line("");
        let clear_field_func = self.clear_field_func();
        w.pub_fn(&format!("{}(&mut self)", clear_field_func), |w| {
            self.write_clear(w);
        });

        if self.has_has() {
            w.write_line("");
            self.write_message_field_has(w);
        }

        w.write_line("");
        self.write_message_field_set(w);

        if self.has_mut() {
            w.write_line("");
            self.write_message_field_mut(w);
        }

        if self.has_take() {
            w.write_line("");
            self.write_message_field_take(w);
        }
    }
}

pub(crate) fn rust_field_name_for_protobuf_field_name(name: &str) -> RustIdent {
    if rust::is_rust_keyword(name) {
        return RustIdent::new(&format!("field_{}", name));
    } else {
        RustIdent::new(name)
    }
}
