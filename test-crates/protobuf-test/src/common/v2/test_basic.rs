use protobuf::descriptor;
use protobuf::reflect;
use protobuf::CodedInputStream;
use protobuf::CodedOutputStream;
use protobuf::EnumFull;
use protobuf::Message;
use protobuf::MessageFull;
use protobuf_test_common::hex::decode_hex;
use protobuf_test_common::*;

use super::test_basic_pb::*;

#[test]
fn test1() {
    let mut test1 = Test1::new();
    test1.set_a(150);
    test_serialize_deserialize_with_dynamic("08 96 01", &test1);
}

#[test]
fn test2() {
    let mut test2 = Test2::new();
    test2.set_b("testing".to_string());
    test_serialize_deserialize_with_dynamic("12 07 74 65 73 74 69 6e 67", &test2);
}

#[test]
fn test3() {
    let mut test1 = Test1::new();
    test1.set_a(150);
    let mut test3 = Test3::new();
    test3.set_c(test1);
    test_serialize_deserialize_with_dynamic("1a 03 08 96 01", &test3);
}

#[test]
fn test4() {
    let mut test4 = Test4::new();
    test4.set_d([3i32, 270, 86942].to_vec());
    test_serialize_deserialize_with_dynamic("22 06 03 8E 02 9E A7 05", &test4);
}

#[test]
fn test_recursion_limit() {
    let mut test = TestRecursion::new();
    for _ in 0..10 {
        let mut t = TestRecursion::new();
        t.mut_children().push(test);
        test = t;
    }

    let bytes = test.write_to_bytes().unwrap();
    let cases = vec![(None, false), (Some(9), true), (Some(10), false)];

    for (limit, has_err) in cases {
        let mut is = CodedInputStream::from_bytes(&bytes);
        if let Some(limit) = limit {
            is.set_recursion_limit(limit);
        }
        let mut t = TestRecursion::new();
        let res = t.merge_from(&mut is);
        assert_eq!(res.is_err(), has_err, "limit: {:?}", limit);
        if !has_err {
            assert_eq!(t, test, "limit: {:?}", limit);
        }
    }
}

#[test]
fn test_end_by_negative_int() {
    // added following https://github.com/stepancheg/rust-protobuf/pull/209
    let mut test = Test1::new();
    test.set_a(-1);
    test_serialize_deserialize_with_dynamic("08 ff ff ff ff ff ff ff ff ff 01", &test);
}

#[test]
fn test_empty() {
    test_serialize_deserialize_with_dynamic("", &TestEmpty::new());
}

#[test]
fn test_read_junk() {
    assert!(Test1::parse_from_bytes(&decode_hex("00")).is_err());
}

#[test]
fn test_unknown_fields_length_delimited() {
    let mut message = TestUnknownFields::new();
    message.set_a(150);
    message
        .mut_unknown_fields()
        .add_length_delimited(4, [0x10u8, 0x20, 0x30].to_vec());
    test_serialize_deserialize_with_dynamic("08 96 01 22 03 10 20 30", &message);
}

#[test]
fn test_unknown_fields_fixed32() {
    let mut message = TestUnknownFields::new();
    message.set_a(150);
    message.mut_unknown_fields().add_fixed32(4, 0x01020304);
    message.mut_unknown_fields().add_fixed32(4, 0xA1A2A3A4);
    test_serialize_deserialize_with_dynamic("08 96 01 25 04 03 02 01 25 A4 A3 A2 A1", &message);
}

#[test]
fn test_types_singular() {
    let mut message = TestTypesSingular::new();
    message.set_double_field(19f64);
    message.set_float_field(20f32);
    message.set_int32_field(21);
    message.set_int64_field(-22);
    message.set_uint32_field(23);
    message.set_uint64_field(24);
    message.set_sint32_field(-25);
    message.set_sint64_field(26);
    message.set_fixed32_field(27);
    message.set_fixed64_field(28);
    message.set_sfixed32_field(-29);
    message.set_sfixed64_field(30);
    message.set_bool_field(true);
    message.set_string_field("thirty two".to_string());
    message.set_bytes_field([33u8, 34].to_vec());
    message.set_enum_field(TestEnumDescriptor::BLUE);
    test_serialize_deserialize_no_hex_with_dynamic(&message);
}

#[test]
fn test_types_repeated() {
    let mut message = TestTypesRepeated::new();
    message.set_double_field([19f64, 20f64].to_vec());
    message.set_float_field([20f32].to_vec());
    message.set_int32_field([21i32, -22, 23].to_vec());
    message.set_int64_field([22i64].to_vec());
    message.set_uint32_field([23u32, 24].to_vec());
    message.set_uint64_field([24u64].to_vec());
    message.set_sint32_field([25i32].to_vec());
    message.set_sint64_field([26i64, -27].to_vec());
    message.set_fixed32_field([27u32].to_vec());
    message.set_fixed64_field([28u64].to_vec());
    message.set_sfixed32_field([29i32, -30].to_vec());
    message.set_sfixed64_field([30i64].to_vec());
    message.set_bool_field([true, true].to_vec());
    message.set_string_field(vec!["thirty two".to_string(), "thirty three".to_string()]);
    message.set_bytes_field(vec![[33u8, 34].to_vec(), [35u8].to_vec()]);
    message.set_enum_field(
        [
            TestEnumDescriptor::BLUE.into(),
            TestEnumDescriptor::GREEN.into(),
        ]
        .to_vec(),
    );
    test_serialize_deserialize_no_hex_with_dynamic(&message);
}

#[test]
fn test_types_repeated_packed() {
    let mut message = TestTypesRepeatedPacked::new();
    message.set_double_field([19f64, 20f64].to_vec());
    message.set_float_field([20f32].to_vec());
    message.set_int32_field([21i32, -22, 23].to_vec());
    message.set_int64_field([22i64].to_vec());
    message.set_uint32_field([23u32, 24].to_vec());
    message.set_uint64_field([24u64].to_vec());
    message.set_sint32_field([-25i32, 26].to_vec());
    message.set_sint64_field([26i64, -27, 28].to_vec());
    message.set_fixed32_field([27u32].to_vec());
    message.set_fixed64_field([28u64].to_vec());
    message.set_sfixed32_field([29i32, -30].to_vec());
    message.set_sfixed64_field([30i64].to_vec());
    message.set_bool_field([true, true].to_vec());
    message.set_string_field(vec!["thirty two".to_string(), "thirty three".to_string()]);
    message.set_bytes_field(vec![[33u8, 34].to_vec(), [35u8].to_vec()]);
    message.set_enum_field(
        [
            TestEnumDescriptor::BLUE.into(),
            TestEnumDescriptor::GREEN.into(),
        ]
        .to_vec(),
    );
    test_serialize_deserialize_no_hex_with_dynamic(&message);
}

#[test]
fn test_file_descriptor_proto() {
    let p: &'static descriptor::FileDescriptorProto = file_descriptor_proto();
    assert!(p.has_name());
    assert_eq!("test_basic_pb.proto", p.name());
}

#[test]
fn test_default_instance() {
    let d = TestDefaultInstance::new();
    assert_eq!("", d.field().s());
}

#[test]
fn test_message_descriptor() {
    assert_eq!("TestDescriptor", TestDescriptor::descriptor_static().name());

    let d = reflect::MessageDescriptor::for_type::<TestDescriptor>();
    assert_eq!("TestDescriptor", d.name());
    assert_eq!("basic.TestDescriptor", d.full_name());

    let mut t = TestDescriptor::new();
    t.set_stuff(55);

    let field = d.field_by_name("stuff").unwrap();
    assert_eq!(
        55,
        field.get_singular_field_or_default(&t).to_i32().unwrap()
    );
}

#[test]
fn test_enum_descriptor() {
    let d = TestEnumDescriptor::RED.enum_descriptor();
    assert_eq!("TestEnumDescriptor", d.name());
    assert_eq!(
        "TestEnumDescriptor",
        reflect::EnumDescriptor::for_type::<TestEnumDescriptor>().name()
    );
    assert_eq!("GREEN", d.value_by_name("GREEN").unwrap().name());
}

#[test]
fn test_invalid_tag() {
    // 01 is invalid tag, because field number for that tag would be 0
    let bytes = decode_hex("01 02 03");
    let r = TestInvalidTag::parse_from_bytes(&bytes);
    assert!(r.is_err());
}

#[test]
fn test_truncated_no_varint() {
    // 08 is valid tag that should be followed by varint
    let bytes = decode_hex("08");
    let r = TestTruncated::parse_from_bytes(&bytes);
    assert!(r.is_err());
}

#[test]
fn test_truncated_middle_of_varint() {
    // 08 is field 1, wire type varint
    // 96 is non-final byte of varint
    let bytes = decode_hex("08 96");
    let r = TestTruncated::parse_from_bytes(&bytes);
    assert!(r.is_err());
}

#[test]
fn test_truncated_middle_of_length_delimited() {
    // 0a is field 1, wire type length delimited
    // 03 is length 3
    let bytes = decode_hex("0a 03 10");
    let r = TestTruncated::parse_from_bytes(&bytes);
    assert!(r.is_err());
}

#[test]
fn test_truncated_repeated_packed() {
    // 12 is field 2, wire type length delimited
    // 04 is length 4
    let bytes = decode_hex("12 04 10 20");
    let r = TestTruncated::parse_from_bytes(&bytes);
    assert!(r.is_err());
}

#[test]
fn test_bug_sint() {
    {
        let mut x = TestBugSint::new();
        x.set_s32(-1);
        test_serialize_deserialize_with_dynamic("08 01", &x);
    }
    {
        let mut x = TestBugSint::new();
        x.set_s64(-2);
        test_serialize_deserialize_with_dynamic("10 03", &x);
    }
}

/// Smoke test which validates that read from the network doesn't block
#[test]
fn test_parse_length_delimited_from_network_smoke() {
    use std::io::Write;
    use std::net;
    use std::thread;

    let listener = net::TcpListener::bind(("127.0.0.1", 0)).expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    thread::spawn(move || {
        let mut test1 = Test1::new();
        test1.set_a(10);
        let bytes = test1.write_length_delimited_to_bytes().expect("bytes");

        let mut stream = listener.accept().expect("accept").0;
        stream.write(&bytes).expect("write");
    });

    let mut tcp_stream = net::TcpStream::connect(addr).expect("connect");
    let mut is = CodedInputStream::new(&mut tcp_stream);
    let test1: Test1 = is.read_message().expect("read_message");
    assert_eq!(10, test1.a());
    is.check_eof().expect("check_eof");
}

/// Test if providing a smaller buffer, protobuf can detect and report error.
#[test]
fn test_serialize_too_large_message() {
    let mut test1 = Test1::new();
    test1.set_a(150);
    let len = test1.compute_size();
    let mut bytes = vec![0; len as usize - 1];
    let mut s = CodedOutputStream::bytes(&mut bytes);
    test1.write_to_with_cached_sizes(&mut s).unwrap_err();
}
