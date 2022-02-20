// This file is generated by rust-protobuf 3.0.0-alpha.7. Do not edit
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

//! Generated file from `google/protobuf/timestamp.proto`

#[derive(PartialEq,Clone,Default,Debug)]
// @@protoc_insertion_point(message:google.protobuf.Timestamp)
pub struct Timestamp {
    // message fields
    ///  Represents seconds of UTC time since Unix epoch
    ///  1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    ///  9999-12-31T23:59:59Z inclusive.
    // @@protoc_insertion_point(field:google.protobuf.Timestamp.seconds)
    pub seconds: i64,
    ///  Non-negative fractions of a second at nanosecond resolution. Negative
    ///  second values with fractions must still have non-negative nanos values
    ///  that count forward in time. Must be from 0 to 999,999,999
    ///  inclusive.
    // @@protoc_insertion_point(field:google.protobuf.Timestamp.nanos)
    pub nanos: i32,
    // special fields
    // @@protoc_insertion_point(special_field:google.protobuf.Timestamp.unknown_fields)
    pub unknown_fields: crate::UnknownFields,
    // @@protoc_insertion_point(special_field:google.protobuf.Timestamp.cached_size)
    pub cached_size: crate::rt::CachedSize,
}

impl<'a> ::std::default::Default for &'a Timestamp {
    fn default() -> &'a Timestamp {
        <Timestamp as crate::Message>::default_instance()
    }
}

impl Timestamp {
    pub fn new() -> Timestamp {
        ::std::default::Default::default()
    }

    fn generated_message_descriptor_data() -> crate::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(2);
        fields.push(crate::reflect::rt::v2::make_simpler_field_accessor::<_, _>(
            "seconds",
            |m: &Timestamp| { &m.seconds },
            |m: &mut Timestamp| { &mut m.seconds },
        ));
        fields.push(crate::reflect::rt::v2::make_simpler_field_accessor::<_, _>(
            "nanos",
            |m: &Timestamp| { &m.nanos },
            |m: &mut Timestamp| { &mut m.nanos },
        ));
        crate::reflect::GeneratedMessageDescriptorData::new_2::<Timestamp>(
            "Timestamp",
            0,
            fields,
        )
    }
}

impl crate::Message for Timestamp {
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

    fn new() -> Timestamp {
        Timestamp::new()
    }

    fn default_instance() -> &'static Timestamp {
        static instance: Timestamp = Timestamp {
            seconds: 0,
            nanos: 0,
            unknown_fields: crate::UnknownFields::new(),
            cached_size: crate::rt::CachedSize::new(),
        };
        &instance
    }
}

impl crate::MessageFull for Timestamp {

    fn descriptor_static() -> crate::reflect::MessageDescriptor {
        crate::reflect::MessageDescriptor::new_generated_2(file_descriptor(), 0)
    }
}

impl crate::Clear for Timestamp {
    fn clear(&mut self) {
        self.seconds = 0;
        self.nanos = 0;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        crate::text_format::fmt(self, f)
    }
}

impl crate::reflect::ProtobufValue for Timestamp {
    type RuntimeType = crate::reflect::runtime_types::RuntimeTypeMessage<Self>;
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x1fgoogle/protobuf/timestamp.proto\x12\x0fgoogle.protobuf\";\n\tTimes\
    tamp\x12\x18\n\x07seconds\x18\x01\x20\x01(\x03R\x07seconds\x12\x14\n\x05\
    nanos\x18\x02\x20\x01(\x05R\x05nanosB\x85\x01\n\x13com.google.protobufB\
    \x0eTimestampProtoP\x01Z2google.golang.org/protobuf/types/known/timestam\
    ppb\xf8\x01\x01\xa2\x02\x03GPB\xaa\x02\x1eGoogle.Protobuf.WellKnownTypes\
    J\xc5/\n\x07\x12\x05\x1e\0\x92\x01\x01\n\xcc\x0c\n\x01\x0c\x12\x03\x1e\0\
    \x122\xc1\x0c\x20Protocol\x20Buffers\x20-\x20Google's\x20data\x20interch\
    ange\x20format\n\x20Copyright\x202008\x20Google\x20Inc.\x20\x20All\x20ri\
    ghts\x20reserved.\n\x20https://developers.google.com/protocol-buffers/\n\
    \n\x20Redistribution\x20and\x20use\x20in\x20source\x20and\x20binary\x20f\
    orms,\x20with\x20or\x20without\n\x20modification,\x20are\x20permitted\
    \x20provided\x20that\x20the\x20following\x20conditions\x20are\n\x20met:\
    \n\n\x20\x20\x20\x20\x20*\x20Redistributions\x20of\x20source\x20code\x20\
    must\x20retain\x20the\x20above\x20copyright\n\x20notice,\x20this\x20list\
    \x20of\x20conditions\x20and\x20the\x20following\x20disclaimer.\n\x20\x20\
    \x20\x20\x20*\x20Redistributions\x20in\x20binary\x20form\x20must\x20repr\
    oduce\x20the\x20above\n\x20copyright\x20notice,\x20this\x20list\x20of\
    \x20conditions\x20and\x20the\x20following\x20disclaimer\n\x20in\x20the\
    \x20documentation\x20and/or\x20other\x20materials\x20provided\x20with\
    \x20the\n\x20distribution.\n\x20\x20\x20\x20\x20*\x20Neither\x20the\x20n\
    ame\x20of\x20Google\x20Inc.\x20nor\x20the\x20names\x20of\x20its\n\x20con\
    tributors\x20may\x20be\x20used\x20to\x20endorse\x20or\x20promote\x20prod\
    ucts\x20derived\x20from\n\x20this\x20software\x20without\x20specific\x20\
    prior\x20written\x20permission.\n\n\x20THIS\x20SOFTWARE\x20IS\x20PROVIDE\
    D\x20BY\x20THE\x20COPYRIGHT\x20HOLDERS\x20AND\x20CONTRIBUTORS\n\x20\"AS\
    \x20IS\"\x20AND\x20ANY\x20EXPRESS\x20OR\x20IMPLIED\x20WARRANTIES,\x20INC\
    LUDING,\x20BUT\x20NOT\n\x20LIMITED\x20TO,\x20THE\x20IMPLIED\x20WARRANTIE\
    S\x20OF\x20MERCHANTABILITY\x20AND\x20FITNESS\x20FOR\n\x20A\x20PARTICULAR\
    \x20PURPOSE\x20ARE\x20DISCLAIMED.\x20IN\x20NO\x20EVENT\x20SHALL\x20THE\
    \x20COPYRIGHT\n\x20OWNER\x20OR\x20CONTRIBUTORS\x20BE\x20LIABLE\x20FOR\
    \x20ANY\x20DIRECT,\x20INDIRECT,\x20INCIDENTAL,\n\x20SPECIAL,\x20EXEMPLAR\
    Y,\x20OR\x20CONSEQUENTIAL\x20DAMAGES\x20(INCLUDING,\x20BUT\x20NOT\n\x20L\
    IMITED\x20TO,\x20PROCUREMENT\x20OF\x20SUBSTITUTE\x20GOODS\x20OR\x20SERVI\
    CES;\x20LOSS\x20OF\x20USE,\n\x20DATA,\x20OR\x20PROFITS;\x20OR\x20BUSINES\
    S\x20INTERRUPTION)\x20HOWEVER\x20CAUSED\x20AND\x20ON\x20ANY\n\x20THEORY\
    \x20OF\x20LIABILITY,\x20WHETHER\x20IN\x20CONTRACT,\x20STRICT\x20LIABILIT\
    Y,\x20OR\x20TORT\n\x20(INCLUDING\x20NEGLIGENCE\x20OR\x20OTHERWISE)\x20AR\
    ISING\x20IN\x20ANY\x20WAY\x20OUT\x20OF\x20THE\x20USE\n\x20OF\x20THIS\x20\
    SOFTWARE,\x20EVEN\x20IF\x20ADVISED\x20OF\x20THE\x20POSSIBILITY\x20OF\x20\
    SUCH\x20DAMAGE.\n\n\x08\n\x01\x02\x12\x03\x20\0\x18\n\x08\n\x01\x08\x12\
    \x03\"\0;\n\t\n\x02\x08%\x12\x03\"\0;\n\x08\n\x01\x08\x12\x03#\0\x1f\n\t\
    \n\x02\x08\x1f\x12\x03#\0\x1f\n\x08\n\x01\x08\x12\x03$\0I\n\t\n\x02\x08\
    \x0b\x12\x03$\0I\n\x08\n\x01\x08\x12\x03%\0,\n\t\n\x02\x08\x01\x12\x03%\
    \0,\n\x08\n\x01\x08\x12\x03&\0/\n\t\n\x02\x08\x08\x12\x03&\0/\n\x08\n\
    \x01\x08\x12\x03'\0\"\n\t\n\x02\x08\n\x12\x03'\0\"\n\x08\n\x01\x08\x12\
    \x03(\0!\n\t\n\x02\x08$\x12\x03(\0!\n\xde\x1d\n\x02\x04\0\x12\x06\x87\
    \x01\0\x92\x01\x01\x1a\xcf\x1d\x20A\x20Timestamp\x20represents\x20a\x20p\
    oint\x20in\x20time\x20independent\x20of\x20any\x20time\x20zone\x20or\x20\
    local\n\x20calendar,\x20encoded\x20as\x20a\x20count\x20of\x20seconds\x20\
    and\x20fractions\x20of\x20seconds\x20at\n\x20nanosecond\x20resolution.\
    \x20The\x20count\x20is\x20relative\x20to\x20an\x20epoch\x20at\x20UTC\x20\
    midnight\x20on\n\x20January\x201,\x201970,\x20in\x20the\x20proleptic\x20\
    Gregorian\x20calendar\x20which\x20extends\x20the\n\x20Gregorian\x20calen\
    dar\x20backwards\x20to\x20year\x20one.\n\n\x20All\x20minutes\x20are\x206\
    0\x20seconds\x20long.\x20Leap\x20seconds\x20are\x20\"smeared\"\x20so\x20\
    that\x20no\x20leap\n\x20second\x20table\x20is\x20needed\x20for\x20interp\
    retation,\x20using\x20a\x20[24-hour\x20linear\n\x20smear](https://develo\
    pers.google.com/time/smear).\n\n\x20The\x20range\x20is\x20from\x200001-0\
    1-01T00:00:00Z\x20to\x209999-12-31T23:59:59.999999999Z.\x20By\n\x20restr\
    icting\x20to\x20that\x20range,\x20we\x20ensure\x20that\x20we\x20can\x20c\
    onvert\x20to\x20and\x20from\x20[RFC\n\x203339](https://www.ietf.org/rfc/\
    rfc3339.txt)\x20date\x20strings.\n\n\x20#\x20Examples\n\n\x20Example\x20\
    1:\x20Compute\x20Timestamp\x20from\x20POSIX\x20`time()`.\n\n\x20\x20\x20\
    \x20\x20Timestamp\x20timestamp;\n\x20\x20\x20\x20\x20timestamp.set_secon\
    ds(time(NULL));\n\x20\x20\x20\x20\x20timestamp.set_nanos(0);\n\n\x20Exam\
    ple\x202:\x20Compute\x20Timestamp\x20from\x20POSIX\x20`gettimeofday()`.\
    \n\n\x20\x20\x20\x20\x20struct\x20timeval\x20tv;\n\x20\x20\x20\x20\x20ge\
    ttimeofday(&tv,\x20NULL);\n\n\x20\x20\x20\x20\x20Timestamp\x20timestamp;\
    \n\x20\x20\x20\x20\x20timestamp.set_seconds(tv.tv_sec);\n\x20\x20\x20\
    \x20\x20timestamp.set_nanos(tv.tv_usec\x20*\x201000);\n\n\x20Example\x20\
    3:\x20Compute\x20Timestamp\x20from\x20Win32\x20`GetSystemTimeAsFileTime(\
    )`.\n\n\x20\x20\x20\x20\x20FILETIME\x20ft;\n\x20\x20\x20\x20\x20GetSyste\
    mTimeAsFileTime(&ft);\n\x20\x20\x20\x20\x20UINT64\x20ticks\x20=\x20(((UI\
    NT64)ft.dwHighDateTime)\x20<<\x2032)\x20|\x20ft.dwLowDateTime;\n\n\x20\
    \x20\x20\x20\x20//\x20A\x20Windows\x20tick\x20is\x20100\x20nanoseconds.\
    \x20Windows\x20epoch\x201601-01-01T00:00:00Z\n\x20\x20\x20\x20\x20//\x20\
    is\x2011644473600\x20seconds\x20before\x20Unix\x20epoch\x201970-01-01T00\
    :00:00Z.\n\x20\x20\x20\x20\x20Timestamp\x20timestamp;\n\x20\x20\x20\x20\
    \x20timestamp.set_seconds((INT64)\x20((ticks\x20/\x2010000000)\x20-\x201\
    1644473600LL));\n\x20\x20\x20\x20\x20timestamp.set_nanos((INT32)\x20((ti\
    cks\x20%\x2010000000)\x20*\x20100));\n\n\x20Example\x204:\x20Compute\x20\
    Timestamp\x20from\x20Java\x20`System.currentTimeMillis()`.\n\n\x20\x20\
    \x20\x20\x20long\x20millis\x20=\x20System.currentTimeMillis();\n\n\x20\
    \x20\x20\x20\x20Timestamp\x20timestamp\x20=\x20Timestamp.newBuilder().se\
    tSeconds(millis\x20/\x201000)\n\x20\x20\x20\x20\x20\x20\x20\x20\x20.setN\
    anos((int)\x20((millis\x20%\x201000)\x20*\x201000000)).build();\n\n\n\
    \x20Example\x205:\x20Compute\x20Timestamp\x20from\x20Java\x20`Instant.no\
    w()`.\n\n\x20\x20\x20\x20\x20Instant\x20now\x20=\x20Instant.now();\n\n\
    \x20\x20\x20\x20\x20Timestamp\x20timestamp\x20=\n\x20\x20\x20\x20\x20\
    \x20\x20\x20\x20Timestamp.newBuilder().setSeconds(now.getEpochSecond())\
    \n\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20.setNanos(now.getN\
    ano()).build();\n\n\n\x20Example\x206:\x20Compute\x20Timestamp\x20from\
    \x20current\x20time\x20in\x20Python.\n\n\x20\x20\x20\x20\x20timestamp\
    \x20=\x20Timestamp()\n\x20\x20\x20\x20\x20timestamp.GetCurrentTime()\n\n\
    \x20#\x20JSON\x20Mapping\n\n\x20In\x20JSON\x20format,\x20the\x20Timestam\
    p\x20type\x20is\x20encoded\x20as\x20a\x20string\x20in\x20the\n\x20[RFC\
    \x203339](https://www.ietf.org/rfc/rfc3339.txt)\x20format.\x20That\x20is\
    ,\x20the\n\x20format\x20is\x20\"{year}-{month}-{day}T{hour}:{min}:{sec}[\
    .{frac_sec}]Z\"\n\x20where\x20{year}\x20is\x20always\x20expressed\x20usi\
    ng\x20four\x20digits\x20while\x20{month},\x20{day},\n\x20{hour},\x20{min\
    },\x20and\x20{sec}\x20are\x20zero-padded\x20to\x20two\x20digits\x20each.\
    \x20The\x20fractional\n\x20seconds,\x20which\x20can\x20go\x20up\x20to\
    \x209\x20digits\x20(i.e.\x20up\x20to\x201\x20nanosecond\x20resolution),\
    \n\x20are\x20optional.\x20The\x20\"Z\"\x20suffix\x20indicates\x20the\x20\
    timezone\x20(\"UTC\");\x20the\x20timezone\n\x20is\x20required.\x20A\x20p\
    roto3\x20JSON\x20serializer\x20should\x20always\x20use\x20UTC\x20(as\x20\
    indicated\x20by\n\x20\"Z\")\x20when\x20printing\x20the\x20Timestamp\x20t\
    ype\x20and\x20a\x20proto3\x20JSON\x20parser\x20should\x20be\n\x20able\
    \x20to\x20accept\x20both\x20UTC\x20and\x20other\x20timezones\x20(as\x20i\
    ndicated\x20by\x20an\x20offset).\n\n\x20For\x20example,\x20\"2017-01-15T\
    01:30:15.01Z\"\x20encodes\x2015.01\x20seconds\x20past\n\x2001:30\x20UTC\
    \x20on\x20January\x2015,\x202017.\n\n\x20In\x20JavaScript,\x20one\x20can\
    \x20convert\x20a\x20Date\x20object\x20to\x20this\x20format\x20using\x20t\
    he\n\x20standard\n\x20[toISOString()](https://developer.mozilla.org/en-U\
    S/docs/Web/JavaScript/Reference/Global_Objects/Date/toISOString)\n\x20me\
    thod.\x20In\x20Python,\x20a\x20standard\x20`datetime.datetime`\x20object\
    \x20can\x20be\x20converted\n\x20to\x20this\x20format\x20using\n\x20[`str\
    ftime`](https://docs.python.org/2/library/time.html#time.strftime)\x20wi\
    th\n\x20the\x20time\x20format\x20spec\x20'%Y-%m-%dT%H:%M:%S.%fZ'.\x20Lik\
    ewise,\x20in\x20Java,\x20one\x20can\x20use\n\x20the\x20Joda\x20Time's\
    \x20[`ISODateTimeFormat.dateTime()`](\n\x20http://www.joda.org/joda-time\
    /apidocs/org/joda/time/format/ISODateTimeFormat.html#dateTime%2D%2D\n\
    \x20)\x20to\x20obtain\x20a\x20formatter\x20capable\x20of\x20generating\
    \x20timestamps\x20in\x20this\x20format.\n\n\n\n\x0b\n\x03\x04\0\x01\x12\
    \x04\x87\x01\x08\x11\n\x9d\x01\n\x04\x04\0\x02\0\x12\x04\x8b\x01\x02\x14\
    \x1a\x8e\x01\x20Represents\x20seconds\x20of\x20UTC\x20time\x20since\x20U\
    nix\x20epoch\n\x201970-01-01T00:00:00Z.\x20Must\x20be\x20from\x200001-01\
    -01T00:00:00Z\x20to\n\x209999-12-31T23:59:59Z\x20inclusive.\n\n\r\n\x05\
    \x04\0\x02\0\x05\x12\x04\x8b\x01\x02\x07\n\r\n\x05\x04\0\x02\0\x01\x12\
    \x04\x8b\x01\x08\x0f\n\r\n\x05\x04\0\x02\0\x03\x12\x04\x8b\x01\x12\x13\n\
    \xe5\x01\n\x04\x04\0\x02\x01\x12\x04\x91\x01\x02\x12\x1a\xd6\x01\x20Non-\
    negative\x20fractions\x20of\x20a\x20second\x20at\x20nanosecond\x20resolu\
    tion.\x20Negative\n\x20second\x20values\x20with\x20fractions\x20must\x20\
    still\x20have\x20non-negative\x20nanos\x20values\n\x20that\x20count\x20f\
    orward\x20in\x20time.\x20Must\x20be\x20from\x200\x20to\x20999,999,999\n\
    \x20inclusive.\n\n\r\n\x05\x04\0\x02\x01\x05\x12\x04\x91\x01\x02\x07\n\r\
    \n\x05\x04\0\x02\x01\x01\x12\x04\x91\x01\x08\r\n\r\n\x05\x04\0\x02\x01\
    \x03\x12\x04\x91\x01\x10\x11b\x06proto3\
";

/// `FileDescriptorProto` object which was a source for this generated file
pub fn file_descriptor_proto() -> &'static crate::descriptor::FileDescriptorProto {
    static file_descriptor_proto_lazy: crate::rt::LazyV2<crate::descriptor::FileDescriptorProto> = crate::rt::LazyV2::INIT;
    file_descriptor_proto_lazy.get(|| {
        crate::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
    })
}

/// `FileDescriptor` object which allows dynamic access to files
pub fn file_descriptor() -> crate::reflect::FileDescriptor {
    static file_descriptor_lazy: crate::rt::LazyV2<crate::reflect::GeneratedFileDescriptor> = crate::rt::LazyV2::INIT;
    let file_descriptor = file_descriptor_lazy.get(|| {
        let mut deps = ::std::vec::Vec::new();
        let mut messages = ::std::vec::Vec::new();
        messages.push(Timestamp::generated_message_descriptor_data());
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
