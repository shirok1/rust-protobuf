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

//! Generated file from `google/protobuf/duration.proto`

#[derive(PartialEq,Clone,Default,Debug)]
// @@protoc_insertion_point(message:google.protobuf.Duration)
pub struct Duration {
    // message fields
    ///  Signed seconds of the span of time. Must be from -315,576,000,000
    ///  to +315,576,000,000 inclusive. Note: these bounds are computed from:
    ///  60 sec/min * 60 min/hr * 24 hr/day * 365.25 days/year * 10000 years
    // @@protoc_insertion_point(field:google.protobuf.Duration.seconds)
    pub seconds: i64,
    ///  Signed fractions of a second at nanosecond resolution of the span
    ///  of time. Durations less than one second are represented with a 0
    ///  `seconds` field and a positive or negative `nanos` field. For durations
    ///  of one second or more, a non-zero value for the `nanos` field must be
    ///  of the same sign as the `seconds` field. Must be from -999,999,999
    ///  to +999,999,999 inclusive.
    // @@protoc_insertion_point(field:google.protobuf.Duration.nanos)
    pub nanos: i32,
    // special fields
    // @@protoc_insertion_point(special_field:google.protobuf.Duration.unknown_fields)
    pub unknown_fields: crate::UnknownFields,
    // @@protoc_insertion_point(special_field:google.protobuf.Duration.cached_size)
    pub cached_size: crate::rt::CachedSize,
}

impl<'a> ::std::default::Default for &'a Duration {
    fn default() -> &'a Duration {
        <Duration as crate::Message>::default_instance()
    }
}

impl Duration {
    pub fn new() -> Duration {
        ::std::default::Default::default()
    }

    fn generated_message_descriptor_data() -> crate::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(2);
        fields.push(crate::reflect::rt::v2::make_simpler_field_accessor::<_, _>(
            "seconds",
            |m: &Duration| { &m.seconds },
            |m: &mut Duration| { &mut m.seconds },
        ));
        fields.push(crate::reflect::rt::v2::make_simpler_field_accessor::<_, _>(
            "nanos",
            |m: &Duration| { &m.nanos },
            |m: &mut Duration| { &mut m.nanos },
        ));
        crate::reflect::GeneratedMessageDescriptorData::new_2::<Duration>(
            "Duration",
            0,
            fields,
        )
    }
}

impl crate::Message for Duration {
    const NAME: &'static str = "Duration";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut crate::CodedInputStream<'_>) -> crate::Result<()> {
        while !is.eof()? {
            let tag = is.read_raw_varint32()?;
            match tag {
                8 => {
                    self.seconds = is.read_int64()?;
                },
                16 => {
                    self.nanos = is.read_int32()?;
                },
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
        if self.seconds != 0 {
            my_size += crate::rt::value_size(1, self.seconds, crate::rt::WireType::Varint);
        }
        if self.nanos != 0 {
            my_size += crate::rt::value_size(2, self.nanos, crate::rt::WireType::Varint);
        }
        my_size += crate::rt::unknown_fields_size(self.unknown_fields());
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut crate::CodedOutputStream<'_>) -> crate::Result<()> {
        if self.seconds != 0 {
            os.write_int64(1, self.seconds)?;
        }
        if self.nanos != 0 {
            os.write_int32(2, self.nanos)?;
        }
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

    fn new() -> Duration {
        Duration::new()
    }

    fn clear(&mut self) {
        self.seconds = 0;
        self.nanos = 0;
        self.unknown_fields.clear();
    }

    fn default_instance() -> &'static Duration {
        static instance: Duration = Duration {
            seconds: 0,
            nanos: 0,
            unknown_fields: crate::UnknownFields::new(),
            cached_size: crate::rt::CachedSize::new(),
        };
        &instance
    }
}

impl crate::MessageFull for Duration {
    fn descriptor_static() -> crate::reflect::MessageDescriptor {
        crate::reflect::MessageDescriptor::new_generated_2(file_descriptor(), 0)
    }
}

impl ::std::fmt::Display for Duration {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        crate::text_format::fmt(self, f)
    }
}

impl crate::reflect::ProtobufValue for Duration {
    type RuntimeType = crate::reflect::runtime_types::RuntimeTypeMessage<Self>;
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x1egoogle/protobuf/duration.proto\x12\x0fgoogle.protobuf\":\n\x08Dura\
    tion\x12\x18\n\x07seconds\x18\x01\x20\x01(\x03R\x07seconds\x12\x14\n\x05\
    nanos\x18\x02\x20\x01(\x05R\x05nanosB\x83\x01\n\x13com.google.protobufB\
    \rDurationProtoP\x01Z1google.golang.org/protobuf/types/known/durationpb\
    \xf8\x01\x01\xa2\x02\x03GPB\xaa\x02\x1eGoogle.Protobuf.WellKnownTypesJ\
    \xda#\n\x06\x12\x04\x1e\0s\x01\n\xcc\x0c\n\x01\x0c\x12\x03\x1e\0\x122\
    \xc1\x0c\x20Protocol\x20Buffers\x20-\x20Google's\x20data\x20interchange\
    \x20format\n\x20Copyright\x202008\x20Google\x20Inc.\x20\x20All\x20rights\
    \x20reserved.\n\x20https://developers.google.com/protocol-buffers/\n\n\
    \x20Redistribution\x20and\x20use\x20in\x20source\x20and\x20binary\x20for\
    ms,\x20with\x20or\x20without\n\x20modification,\x20are\x20permitted\x20p\
    rovided\x20that\x20the\x20following\x20conditions\x20are\n\x20met:\n\n\
    \x20\x20\x20\x20\x20*\x20Redistributions\x20of\x20source\x20code\x20must\
    \x20retain\x20the\x20above\x20copyright\n\x20notice,\x20this\x20list\x20\
    of\x20conditions\x20and\x20the\x20following\x20disclaimer.\n\x20\x20\x20\
    \x20\x20*\x20Redistributions\x20in\x20binary\x20form\x20must\x20reproduc\
    e\x20the\x20above\n\x20copyright\x20notice,\x20this\x20list\x20of\x20con\
    ditions\x20and\x20the\x20following\x20disclaimer\n\x20in\x20the\x20docum\
    entation\x20and/or\x20other\x20materials\x20provided\x20with\x20the\n\
    \x20distribution.\n\x20\x20\x20\x20\x20*\x20Neither\x20the\x20name\x20of\
    \x20Google\x20Inc.\x20nor\x20the\x20names\x20of\x20its\n\x20contributors\
    \x20may\x20be\x20used\x20to\x20endorse\x20or\x20promote\x20products\x20d\
    erived\x20from\n\x20this\x20software\x20without\x20specific\x20prior\x20\
    written\x20permission.\n\n\x20THIS\x20SOFTWARE\x20IS\x20PROVIDED\x20BY\
    \x20THE\x20COPYRIGHT\x20HOLDERS\x20AND\x20CONTRIBUTORS\n\x20\"AS\x20IS\"\
    \x20AND\x20ANY\x20EXPRESS\x20OR\x20IMPLIED\x20WARRANTIES,\x20INCLUDING,\
    \x20BUT\x20NOT\n\x20LIMITED\x20TO,\x20THE\x20IMPLIED\x20WARRANTIES\x20OF\
    \x20MERCHANTABILITY\x20AND\x20FITNESS\x20FOR\n\x20A\x20PARTICULAR\x20PUR\
    POSE\x20ARE\x20DISCLAIMED.\x20IN\x20NO\x20EVENT\x20SHALL\x20THE\x20COPYR\
    IGHT\n\x20OWNER\x20OR\x20CONTRIBUTORS\x20BE\x20LIABLE\x20FOR\x20ANY\x20D\
    IRECT,\x20INDIRECT,\x20INCIDENTAL,\n\x20SPECIAL,\x20EXEMPLARY,\x20OR\x20\
    CONSEQUENTIAL\x20DAMAGES\x20(INCLUDING,\x20BUT\x20NOT\n\x20LIMITED\x20TO\
    ,\x20PROCUREMENT\x20OF\x20SUBSTITUTE\x20GOODS\x20OR\x20SERVICES;\x20LOSS\
    \x20OF\x20USE,\n\x20DATA,\x20OR\x20PROFITS;\x20OR\x20BUSINESS\x20INTERRU\
    PTION)\x20HOWEVER\x20CAUSED\x20AND\x20ON\x20ANY\n\x20THEORY\x20OF\x20LIA\
    BILITY,\x20WHETHER\x20IN\x20CONTRACT,\x20STRICT\x20LIABILITY,\x20OR\x20T\
    ORT\n\x20(INCLUDING\x20NEGLIGENCE\x20OR\x20OTHERWISE)\x20ARISING\x20IN\
    \x20ANY\x20WAY\x20OUT\x20OF\x20THE\x20USE\n\x20OF\x20THIS\x20SOFTWARE,\
    \x20EVEN\x20IF\x20ADVISED\x20OF\x20THE\x20POSSIBILITY\x20OF\x20SUCH\x20D\
    AMAGE.\n\n\x08\n\x01\x02\x12\x03\x20\0\x18\n\x08\n\x01\x08\x12\x03\"\0;\
    \n\t\n\x02\x08%\x12\x03\"\0;\n\x08\n\x01\x08\x12\x03#\0\x1f\n\t\n\x02\
    \x08\x1f\x12\x03#\0\x1f\n\x08\n\x01\x08\x12\x03$\0H\n\t\n\x02\x08\x0b\
    \x12\x03$\0H\n\x08\n\x01\x08\x12\x03%\0,\n\t\n\x02\x08\x01\x12\x03%\0,\n\
    \x08\n\x01\x08\x12\x03&\0.\n\t\n\x02\x08\x08\x12\x03&\0.\n\x08\n\x01\x08\
    \x12\x03'\0\"\n\t\n\x02\x08\n\x12\x03'\0\"\n\x08\n\x01\x08\x12\x03(\0!\n\
    \t\n\x02\x08$\x12\x03(\0!\n\x9e\x10\n\x02\x04\0\x12\x04f\0s\x01\x1a\x91\
    \x10\x20A\x20Duration\x20represents\x20a\x20signed,\x20fixed-length\x20s\
    pan\x20of\x20time\x20represented\n\x20as\x20a\x20count\x20of\x20seconds\
    \x20and\x20fractions\x20of\x20seconds\x20at\x20nanosecond\n\x20resolutio\
    n.\x20It\x20is\x20independent\x20of\x20any\x20calendar\x20and\x20concept\
    s\x20like\x20\"day\"\n\x20or\x20\"month\".\x20It\x20is\x20related\x20to\
    \x20Timestamp\x20in\x20that\x20the\x20difference\x20between\n\x20two\x20\
    Timestamp\x20values\x20is\x20a\x20Duration\x20and\x20it\x20can\x20be\x20\
    added\x20or\x20subtracted\n\x20from\x20a\x20Timestamp.\x20Range\x20is\
    \x20approximately\x20+-10,000\x20years.\n\n\x20#\x20Examples\n\n\x20Exam\
    ple\x201:\x20Compute\x20Duration\x20from\x20two\x20Timestamps\x20in\x20p\
    seudo\x20code.\n\n\x20\x20\x20\x20\x20Timestamp\x20start\x20=\x20...;\n\
    \x20\x20\x20\x20\x20Timestamp\x20end\x20=\x20...;\n\x20\x20\x20\x20\x20D\
    uration\x20duration\x20=\x20...;\n\n\x20\x20\x20\x20\x20duration.seconds\
    \x20=\x20end.seconds\x20-\x20start.seconds;\n\x20\x20\x20\x20\x20duratio\
    n.nanos\x20=\x20end.nanos\x20-\x20start.nanos;\n\n\x20\x20\x20\x20\x20if\
    \x20(duration.seconds\x20<\x200\x20&&\x20duration.nanos\x20>\x200)\x20{\
    \n\x20\x20\x20\x20\x20\x20\x20duration.seconds\x20+=\x201;\n\x20\x20\x20\
    \x20\x20\x20\x20duration.nanos\x20-=\x201000000000;\n\x20\x20\x20\x20\
    \x20}\x20else\x20if\x20(duration.seconds\x20>\x200\x20&&\x20duration.nan\
    os\x20<\x200)\x20{\n\x20\x20\x20\x20\x20\x20\x20duration.seconds\x20-=\
    \x201;\n\x20\x20\x20\x20\x20\x20\x20duration.nanos\x20+=\x201000000000;\
    \n\x20\x20\x20\x20\x20}\n\n\x20Example\x202:\x20Compute\x20Timestamp\x20\
    from\x20Timestamp\x20+\x20Duration\x20in\x20pseudo\x20code.\n\n\x20\x20\
    \x20\x20\x20Timestamp\x20start\x20=\x20...;\n\x20\x20\x20\x20\x20Duratio\
    n\x20duration\x20=\x20...;\n\x20\x20\x20\x20\x20Timestamp\x20end\x20=\
    \x20...;\n\n\x20\x20\x20\x20\x20end.seconds\x20=\x20start.seconds\x20+\
    \x20duration.seconds;\n\x20\x20\x20\x20\x20end.nanos\x20=\x20start.nanos\
    \x20+\x20duration.nanos;\n\n\x20\x20\x20\x20\x20if\x20(end.nanos\x20<\
    \x200)\x20{\n\x20\x20\x20\x20\x20\x20\x20end.seconds\x20-=\x201;\n\x20\
    \x20\x20\x20\x20\x20\x20end.nanos\x20+=\x201000000000;\n\x20\x20\x20\x20\
    \x20}\x20else\x20if\x20(end.nanos\x20>=\x201000000000)\x20{\n\x20\x20\
    \x20\x20\x20\x20\x20end.seconds\x20+=\x201;\n\x20\x20\x20\x20\x20\x20\
    \x20end.nanos\x20-=\x201000000000;\n\x20\x20\x20\x20\x20}\n\n\x20Example\
    \x203:\x20Compute\x20Duration\x20from\x20datetime.timedelta\x20in\x20Pyt\
    hon.\n\n\x20\x20\x20\x20\x20td\x20=\x20datetime.timedelta(days=3,\x20min\
    utes=10)\n\x20\x20\x20\x20\x20duration\x20=\x20Duration()\n\x20\x20\x20\
    \x20\x20duration.FromTimedelta(td)\n\n\x20#\x20JSON\x20Mapping\n\n\x20In\
    \x20JSON\x20format,\x20the\x20Duration\x20type\x20is\x20encoded\x20as\
    \x20a\x20string\x20rather\x20than\x20an\n\x20object,\x20where\x20the\x20\
    string\x20ends\x20in\x20the\x20suffix\x20\"s\"\x20(indicating\x20seconds\
    )\x20and\n\x20is\x20preceded\x20by\x20the\x20number\x20of\x20seconds,\
    \x20with\x20nanoseconds\x20expressed\x20as\n\x20fractional\x20seconds.\
    \x20For\x20example,\x203\x20seconds\x20with\x200\x20nanoseconds\x20shoul\
    d\x20be\n\x20encoded\x20in\x20JSON\x20format\x20as\x20\"3s\",\x20while\
    \x203\x20seconds\x20and\x201\x20nanosecond\x20should\n\x20be\x20expresse\
    d\x20in\x20JSON\x20format\x20as\x20\"3.000000001s\",\x20and\x203\x20seco\
    nds\x20and\x201\n\x20microsecond\x20should\x20be\x20expressed\x20in\x20J\
    SON\x20format\x20as\x20\"3.000001s\".\n\n\n\n\n\n\x03\x04\0\x01\x12\x03f\
    \x08\x10\n\xdc\x01\n\x04\x04\0\x02\0\x12\x03j\x02\x14\x1a\xce\x01\x20Sig\
    ned\x20seconds\x20of\x20the\x20span\x20of\x20time.\x20Must\x20be\x20from\
    \x20-315,576,000,000\n\x20to\x20+315,576,000,000\x20inclusive.\x20Note:\
    \x20these\x20bounds\x20are\x20computed\x20from:\n\x2060\x20sec/min\x20*\
    \x2060\x20min/hr\x20*\x2024\x20hr/day\x20*\x20365.25\x20days/year\x20*\
    \x2010000\x20years\n\n\x0c\n\x05\x04\0\x02\0\x05\x12\x03j\x02\x07\n\x0c\
    \n\x05\x04\0\x02\0\x01\x12\x03j\x08\x0f\n\x0c\n\x05\x04\0\x02\0\x03\x12\
    \x03j\x12\x13\n\x83\x03\n\x04\x04\0\x02\x01\x12\x03r\x02\x12\x1a\xf5\x02\
    \x20Signed\x20fractions\x20of\x20a\x20second\x20at\x20nanosecond\x20reso\
    lution\x20of\x20the\x20span\n\x20of\x20time.\x20Durations\x20less\x20tha\
    n\x20one\x20second\x20are\x20represented\x20with\x20a\x200\n\x20`seconds\
    `\x20field\x20and\x20a\x20positive\x20or\x20negative\x20`nanos`\x20field\
    .\x20For\x20durations\n\x20of\x20one\x20second\x20or\x20more,\x20a\x20no\
    n-zero\x20value\x20for\x20the\x20`nanos`\x20field\x20must\x20be\n\x20of\
    \x20the\x20same\x20sign\x20as\x20the\x20`seconds`\x20field.\x20Must\x20b\
    e\x20from\x20-999,999,999\n\x20to\x20+999,999,999\x20inclusive.\n\n\x0c\
    \n\x05\x04\0\x02\x01\x05\x12\x03r\x02\x07\n\x0c\n\x05\x04\0\x02\x01\x01\
    \x12\x03r\x08\r\n\x0c\n\x05\x04\0\x02\x01\x03\x12\x03r\x10\x11b\x06proto\
    3\
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
        messages.push(Duration::generated_message_descriptor_data());
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
