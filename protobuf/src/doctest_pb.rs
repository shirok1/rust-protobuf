// This file is generated by rust-protobuf 3.0.0-alpha.8. Do not edit
// .proto file is parsed by protoc --rust-out=...
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_results)]
#![allow(unused_mut)]

//! Generated file from `doctest_pb.proto`

#[derive(PartialEq,Clone,Default,Debug)]
// @@protoc_insertion_point(message:MyMessage)
pub struct MyMessage {
    // special fields
    // @@protoc_insertion_point(special_field:MyMessage.unknown_fields)
    pub unknown_fields: crate::UnknownFields,
    // @@protoc_insertion_point(special_field:MyMessage.cached_size)
    pub cached_size: crate::rt::CachedSize,
}

impl<'a> ::std::default::Default for &'a MyMessage {
    fn default() -> &'a MyMessage {
        <MyMessage as crate::Message>::default_instance()
    }
}

impl MyMessage {
    pub fn new() -> MyMessage {
        ::std::default::Default::default()
    }

    fn generated_message_descriptor_data() -> crate::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(0);
        crate::reflect::GeneratedMessageDescriptorData::new_2::<MyMessage>(
            "MyMessage",
            0,
            fields,
        )
    }
}

impl crate::Message for MyMessage {
    const NAME: &'static str = "MyMessage";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut crate::CodedInputStream<'_>) -> crate::Result<()> {
        while !is.eof()? {
            let tag = is.read_raw_varint32()?;
            match tag {
                tag => {
                    crate::rt::read_unknown_or_skip_group(tag, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        my_size += crate::rt::unknown_fields_size(self.unknown_fields());
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut crate::CodedOutputStream<'_>) -> crate::Result<()> {
        os.write_unknown_fields(self.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn unknown_fields(&self) -> &crate::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut crate::UnknownFields {
        &mut self.unknown_fields
    }

    fn new() -> MyMessage {
        MyMessage::new()
    }

    fn clear(&mut self) {
        self.unknown_fields.clear();
    }

    fn default_instance() -> &'static MyMessage {
        static instance: MyMessage = MyMessage {
            unknown_fields: crate::UnknownFields::new(),
            cached_size: crate::rt::CachedSize::new(),
        };
        &instance
    }
}

impl crate::MessageFull for MyMessage {
    fn descriptor_static() -> crate::reflect::MessageDescriptor {
        crate::reflect::MessageDescriptor::new_generated_2(file_descriptor(), 0)
    }
}

impl ::std::fmt::Display for MyMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        crate::text_format::fmt(self, f)
    }
}

impl crate::reflect::ProtobufValue for MyMessage {
    type RuntimeType = crate::reflect::runtime_types::RuntimeTypeMessage<Self>;
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x10doctest_pb.proto\"\x0b\n\tMyMessageJG\n\x06\x12\x04\x02\0\x05\x01\
    \n%\n\x01\x0c\x12\x03\x02\0\x122\x1b\x20Messages\x20used\x20in\x20doctes\
    ts\n\n\n\n\x02\x04\0\x12\x04\x04\0\x05\x01\n\n\n\x03\x04\0\x01\x12\x03\
    \x04\x08\x11b\x06proto3\
";

/// `FileDescriptorProto` object which was a source for this generated file
pub fn file_descriptor_proto() -> &'static crate::descriptor::FileDescriptorProto {
    static file_descriptor_proto_lazy: crate::rt::Lazy<crate::descriptor::FileDescriptorProto> = crate::rt::Lazy::new();
    file_descriptor_proto_lazy.get(|| {
        crate::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
    })
}

/// `FileDescriptor` object which allows dynamic access to files
pub fn file_descriptor() -> crate::reflect::FileDescriptor {
    static file_descriptor_lazy: crate::rt::Lazy<crate::reflect::GeneratedFileDescriptor> = crate::rt::Lazy::new();
    let file_descriptor = file_descriptor_lazy.get(|| {
        let mut deps = ::std::vec::Vec::new();
        let mut messages = ::std::vec::Vec::new();
        messages.push(MyMessage::generated_message_descriptor_data());
        let mut enums = ::std::vec::Vec::new();
        crate::reflect::GeneratedFileDescriptor::new_generated(
            file_descriptor_proto(),
            deps,
            messages,
            enums,
        )
    });
    crate::reflect::FileDescriptor::new_generated_2(file_descriptor)
}
