//! Oneof-related codegen functions.

use std::collections::HashSet;

use protobuf::descriptor::field_descriptor_proto;
use protobuf::reflect::FieldDescriptor;
use protobuf_parse::ProtobufAbsPath;

use crate::customize::ctx::CustomizeElemCtx;
use crate::customize::Customize;
use crate::gen::code_writer::CodeWriter;
use crate::gen::field::rust_field_name_for_protobuf_field_name;
use crate::gen::field::FieldElem;
use crate::gen::field::FieldGen;
use crate::gen::file_and_mod::FileAndMod;
use crate::gen::inside::protobuf_crate_path;
use crate::gen::message::MessageGen;
use crate::gen::protoc_insertion_point::write_protoc_insertion_point_for_oneof;
use crate::gen::protoc_insertion_point::write_protoc_insertion_point_for_oneof_field;
use crate::gen::rust_name::RustIdent;
use crate::gen::rust_name::RustIdentWithPath;
use crate::gen::rust_name::RustPath;
use crate::gen::rust_types_values::make_path;
use crate::gen::rust_types_values::RustType;
use crate::gen::scope::OneofVariantWithContext;
use crate::gen::scope::OneofWithContext;
use crate::gen::scope::RootScope;
use crate::gen::scope::WithScope;

// oneof one { ... }
#[derive(Clone)]
pub(crate) struct OneofField<'a> {
    pub elem: FieldElem<'a>,
    pub oneof_variant_rust_name: RustIdent,
    pub oneof_field_name: RustIdent,
    pub type_name: RustIdentWithPath,
    pub boxed: bool,
}

impl<'a> OneofField<'a> {
    // Detecting recursion: if oneof fields contains a self-reference
    // or another message which has a reference to self,
    // put oneof variant into a box.
    fn need_boxed(
        field: &FieldDescriptor,
        root_scope: &RootScope,
        owner_name: &ProtobufAbsPath,
    ) -> bool {
        let mut visited_messages = HashSet::new();
        let mut fields = vec![field.clone()];
        while let Some(field) = fields.pop() {
            if field.proto().field_type() == field_descriptor_proto::Type::TYPE_MESSAGE {
                let message_name = ProtobufAbsPath::from(field.proto().type_name());
                if !visited_messages.insert(message_name.clone()) {
                    continue;
                }
                if message_name == *owner_name {
                    return true;
                }
                let message = root_scope.find_message(&message_name);
                fields.extend(
                    message
                        .message
                        .fields()
                        .into_iter()
                        .filter(|f| f.containing_oneof().is_some()),
                );
            }
        }
        false
    }

    pub fn parse(
        oneof: &OneofWithContext<'a>,
        field: &FieldDescriptor,
        elem: FieldElem<'a>,
        root_scope: &RootScope,
    ) -> OneofField<'a> {
        let boxed = OneofField::need_boxed(field, root_scope, &oneof.message.name_absolute());

        OneofField {
            elem,
            type_name: oneof.rust_name(),
            boxed,
            oneof_variant_rust_name: rust_field_name_for_protobuf_field_name(field.name()),
            oneof_field_name: oneof.field_name(),
        }
    }

    pub fn rust_type(&self, reference: &FileAndMod) -> RustType {
        let t = self.elem.rust_storage_elem_type(reference);

        if self.boxed {
            RustType::Uniq(Box::new(t))
        } else {
            t
        }
    }

    pub fn variant_path(&self, reference: &RustPath) -> RustIdentWithPath {
        make_path(
            reference,
            &self
                .type_name
                .to_path()
                .with_ident(self.oneof_variant_rust_name.clone()),
        )
    }
}

#[derive(Clone)]
pub(crate) struct OneofVariantGen<'a> {
    oneof: &'a OneofGen<'a>,
    _variant: OneofVariantWithContext<'a>,
    oneof_field: OneofField<'a>,
    pub field: FieldGen<'a>,
    _path: String,
}

impl<'a> OneofVariantGen<'a> {
    fn parse(
        oneof: &'a OneofGen<'a>,
        variant: OneofVariantWithContext<'a>,
        field: &'a FieldGen,
        _root_scope: &RootScope,
    ) -> OneofVariantGen<'a> {
        OneofVariantGen {
            oneof,
            _variant: variant.clone(),
            field: field.clone(),
            _path: format!(
                "{}::{}",
                oneof.type_name_relative(
                    &oneof
                        .oneof
                        .message
                        .scope
                        .rust_path_to_file()
                        .clone()
                        .into_path()
                ),
                field.rust_name
            ),
            oneof_field: OneofField::parse(
                variant.oneof,
                &field.proto_field.field,
                field.elem().clone(),
                oneof.message.root_scope,
            ),
        }
    }

    pub fn rust_type(&self, reference: &FileAndMod) -> RustType {
        self.oneof_field.rust_type(reference)
    }

    pub fn path(&self, reference: &FileAndMod) -> RustPath {
        RustPath::from(format!(
            "{}::{}",
            self.oneof
                .type_name_relative(&reference.relative_mod.clone().into_path()),
            self.field.rust_name
        ))
    }
}

pub(crate) struct OneofGen<'a> {
    // Message containing this oneof
    message: &'a MessageGen<'a>,
    pub oneof: OneofWithContext<'a>,
    customize: CustomizeElemCtx<'a>,
}

impl<'a> OneofGen<'a> {
    pub fn parse(
        message: &'a MessageGen,
        oneof: OneofWithContext<'a>,
        parent_customize: &CustomizeElemCtx<'a>,
    ) -> OneofGen<'a> {
        let customize = parent_customize.child(&Customize::default(), &oneof.oneof);
        OneofGen {
            message,
            oneof,
            customize,
        }
    }

    pub fn type_name_relative(&self, source: &RustPath) -> RustIdentWithPath {
        make_path(source, &self.oneof.rust_name())
    }

    pub fn variants_except_group(&'a self) -> Vec<OneofVariantGen<'a>> {
        self.oneof
            .variants()
            .into_iter()
            .filter_map(|v| {
                let field = self
                    .message
                    .fields
                    .iter()
                    .filter(|f| f.proto_field.name() == v.field.name())
                    .next()
                    .expect(&format!("field not found by name: {}", v.field.name()));
                match field.proto_type {
                    field_descriptor_proto::Type::TYPE_GROUP => None,
                    _ => Some(OneofVariantGen::parse(
                        self,
                        v,
                        field,
                        self.message.root_scope,
                    )),
                }
            })
            .collect()
    }

    pub fn full_storage_type(&self) -> RustType {
        RustType::Option(Box::new(RustType::Oneof(
            self.type_name_relative(
                &self
                    .oneof
                    .message
                    .scope
                    .file_and_mod(self.customize.for_elem.clone())
                    .relative_mod
                    .into_path(),
            )
            .clone(),
        )))
    }

    fn file_and_mod(&self) -> FileAndMod {
        let mut file_and_mod = self
            .message
            .message
            .scope
            .file_and_mod(self.customize.for_elem.clone());
        file_and_mod
            .relative_mod
            .push_ident(self.message.message.mod_name());
        file_and_mod
    }

    fn write_enum(&self, w: &mut CodeWriter) {
        let derive = vec!["Clone", "PartialEq", "Debug"];
        w.derive(&derive);
        w.write_line("#[non_exhaustive]");
        write_protoc_insertion_point_for_oneof(w, &self.customize.for_elem, &self.oneof.oneof);
        w.pub_enum(&self.oneof.rust_name().ident.to_string(), |w| {
            for variant in self.variants_except_group() {
                write_protoc_insertion_point_for_oneof_field(
                    w,
                    &self.customize.for_children,
                    &variant.field.proto_field.field,
                );
                w.write_line(&format!(
                    "{}({}),",
                    variant.field.rust_name,
                    &variant
                        .rust_type(&self.file_and_mod())
                        .to_code(&self.customize.for_elem)
                ));
            }
        });
    }

    fn write_impl_oneof(&self, w: &mut CodeWriter) {
        w.impl_for_block(
            &format!("{}::Oneof", protobuf_crate_path(&self.customize.for_elem)),
            self.oneof.rust_name().ident.to_string(),
            |_w| {
                // nothing here yet
            },
        );
    }

    pub fn write(&self, w: &mut CodeWriter) {
        self.write_enum(w);
        w.write_line("");
        self.write_impl_oneof(w);
    }
}
