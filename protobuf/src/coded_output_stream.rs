use std::fmt;
use std::io;
use std::io::Write;
use std::mem::MaybeUninit;
use std::ptr;
use std::slice;

use crate::error::ProtobufError;
use crate::misc::maybe_uninit_write_slice;
use crate::misc::vec_spare_capacity_mut;
use crate::rt::vec_packed_enum_or_unknown_data_size;
use crate::rt::vec_packed_fixed_data_size;
use crate::rt::vec_packed_varint_data_size;
use crate::rt::vec_packed_varint_zigzag_data_size;
use crate::varint;
use crate::wire_format;
use crate::wire_format::check_message_size;
use crate::wire_format::WireType;
use crate::zigzag::encode_zig_zag_32;
use crate::zigzag::encode_zig_zag_64;
use crate::Enum;
use crate::EnumOrUnknown;
use crate::Message;
use crate::MessageDyn;
use crate::MessageFull;
use crate::UnknownFields;
use crate::UnknownValueRef;

// Equal to the default buffer size of `BufWriter`, so when
// `CodedOutputStream` wraps `BufWriter`, it often skips double buffering.
const OUTPUT_STREAM_BUFFER_SIZE: usize = 8 * 1024;

pub trait WithCodedOutputStream {
    fn with_coded_output_stream<T, F>(self, cb: F) -> crate::Result<T>
    where
        F: FnOnce(&mut CodedOutputStream) -> crate::Result<T>;
}

impl<'a> WithCodedOutputStream for &'a mut (dyn Write + 'a) {
    fn with_coded_output_stream<T, F>(self, cb: F) -> crate::Result<T>
    where
        F: FnOnce(&mut CodedOutputStream) -> crate::Result<T>,
    {
        let mut os = CodedOutputStream::new(self);
        let r = cb(&mut os)?;
        os.flush()?;
        Ok(r)
    }
}

impl<'a> WithCodedOutputStream for &'a mut Vec<u8> {
    fn with_coded_output_stream<T, F>(mut self, cb: F) -> crate::Result<T>
    where
        F: FnOnce(&mut CodedOutputStream) -> crate::Result<T>,
    {
        let mut os = CodedOutputStream::vec(&mut self);
        let r = cb(&mut os)?;
        os.flush()?;
        Ok(r)
    }
}

/// Output buffer/writer for `CodedOutputStream`.
enum OutputTarget<'a> {
    Write(&'a mut dyn Write, Vec<u8>),
    Vec(&'a mut Vec<u8>),
    /// The buffer is passed as `&[u8]` to `CodedOutputStream` constructor
    /// and immediately converted to `buffer` field of `CodedOutputStream`,
    /// it is not needed to be stored here.
    /// Lifetime parameter of `CodedOutputStream` guarantees the buffer is valid
    /// during the lifetime of `CodedOutputStream`.
    Bytes,
}

impl<'a> fmt::Debug for OutputTarget<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputTarget::Write(_w, vec) => f
                .debug_struct("Write")
                .field("buf_len", &vec.len())
                .field("buf_cap", &vec.capacity())
                .finish_non_exhaustive(),
            OutputTarget::Vec(vec) => f
                .debug_struct("Vec")
                .field("len", &vec.len())
                .field("cap", &vec.capacity())
                .finish_non_exhaustive(),
            OutputTarget::Bytes => f.debug_tuple("Bytes").finish(),
        }
    }
}

/// Buffered write with handy utilities
#[derive(Debug)]
pub struct CodedOutputStream<'a> {
    target: OutputTarget<'a>,
    // Actual buffer is owned by `OutputTarget`,
    // and here we alias the buffer so access to the buffer is branchless:
    // access does not require switch by actual target type: `&[], `Vec`, `Write` etc.
    // We don't access the actual buffer in `OutputTarget` except when
    // we initialize `buffer` field here.
    buffer: *mut [MaybeUninit<u8>],
    /// Position within the buffer.
    /// Always correct.
    pos_within_buf: usize,
    /// Absolute position of the buffer start.
    pos_of_buffer_start: u64,
}

impl<'a> CodedOutputStream<'a> {
    /// Construct from given `Write`.
    ///
    /// `CodedOutputStream` is buffered even if `Write` is not
    pub fn new(writer: &'a mut dyn Write) -> CodedOutputStream<'a> {
        let buffer_len = OUTPUT_STREAM_BUFFER_SIZE;

        let mut buffer_storage = Vec::with_capacity(buffer_len);

        // SAFETY: we are not using the `buffer_storage`
        // except for initializing the `buffer` field.
        // See `buffer` field documentation.
        let buffer = vec_spare_capacity_mut(&mut buffer_storage);
        let buffer: *mut [MaybeUninit<u8>] = buffer;

        CodedOutputStream {
            target: OutputTarget::Write(writer, buffer_storage),
            buffer,
            pos_within_buf: 0,
            pos_of_buffer_start: 0,
        }
    }

    /// `CodedOutputStream` which writes directly to bytes.
    ///
    /// Attempt to write more than bytes capacity results in error.
    pub fn bytes(bytes: &'a mut [u8]) -> CodedOutputStream<'a> {
        // SAFETY: it is safe to cast from &mut [u8] to &mut [MaybeUninit<u8>].
        let buffer =
            ptr::slice_from_raw_parts_mut(bytes.as_mut_ptr() as *mut MaybeUninit<u8>, bytes.len());
        CodedOutputStream {
            target: OutputTarget::Bytes,
            buffer,
            pos_within_buf: 0,
            pos_of_buffer_start: 0,
        }
    }

    /// `CodedOutputStream` which writes directly to `Vec<u8>`.
    pub fn vec(vec: &'a mut Vec<u8>) -> CodedOutputStream<'a> {
        let buffer: *mut [MaybeUninit<u8>] = &mut [];
        CodedOutputStream {
            target: OutputTarget::Vec(vec),
            buffer,
            pos_within_buf: 0,
            pos_of_buffer_start: 0,
        }
    }

    /// Total number of bytes written to this stream.
    ///
    /// This number may be larger than the actual number of bytes written to the underlying stream,
    /// if the buffer was not flushed.
    ///
    /// The number may be inaccurate if there was an error during the write.
    pub fn total_bytes_written(&self) -> u64 {
        self.pos_of_buffer_start + self.pos_within_buf as u64
    }

    /// Check if EOF is reached.
    ///
    /// # Panics
    ///
    /// If underlying write has no EOF
    pub fn check_eof(&self) {
        match self.target {
            OutputTarget::Bytes => {
                assert_eq!(self.buffer().len() as u64, self.pos_within_buf as u64);
            }
            OutputTarget::Write(..) | OutputTarget::Vec(..) => {
                panic!("must not be called with Writer or Vec");
            }
        }
    }

    #[inline(always)]
    fn buffer(&self) -> &[MaybeUninit<u8>] {
        // SAFETY: see the `buffer` field documentation about invariants.
        unsafe { &*(self.buffer as *mut [MaybeUninit<u8>]) }
    }

    #[inline(always)]
    fn filled_buffer_impl<'s>(buffer: *mut [MaybeUninit<u8>], position: usize) -> &'s [u8] {
        // SAFETY: this function is safe assuming `buffer` and `position`
        //   are `self.buffer` and `safe.position`:
        //   * `CodedOutputStream` has invariant that `position <= buffer.len()`.
        //   * `buffer` is filled up to `position`.
        unsafe { slice::from_raw_parts_mut(buffer as *mut u8, position) }
    }

    fn refresh_buffer(&mut self) -> crate::Result<()> {
        match self.target {
            OutputTarget::Write(ref mut write, _) => {
                write.write_all(Self::filled_buffer_impl(self.buffer, self.pos_within_buf))?;
                self.pos_of_buffer_start += self.pos_within_buf as u64;
                self.pos_within_buf = 0;
            }
            OutputTarget::Vec(ref mut vec) => unsafe {
                let vec_len = vec.len();
                assert!(vec_len + self.pos_within_buf <= vec.capacity());
                vec.set_len(vec_len + self.pos_within_buf);
                vec.reserve(1);
                self.buffer = vec_spare_capacity_mut(vec);
                self.pos_of_buffer_start += self.pos_within_buf as u64;
                self.pos_within_buf = 0;
            },
            OutputTarget::Bytes => {
                return Err(ProtobufError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    "given slice is too small to serialize the message",
                ))
                .into());
            }
        }
        Ok(())
    }

    /// Flush to buffer to the underlying buffer.
    /// Note that `CodedOutputStream` does `flush` in the destructor,
    /// however, if `flush` in destructor fails, then destructor panics
    /// and program terminates. So it's advisable to explicitly call flush
    /// before destructor.
    pub fn flush(&mut self) -> crate::Result<()> {
        match &mut self.target {
            OutputTarget::Bytes => Ok(()),
            OutputTarget::Vec(vec) => {
                let vec_len = vec.len();
                assert!(vec_len + self.pos_within_buf <= vec.capacity());
                unsafe {
                    vec.set_len(vec_len + self.pos_within_buf);
                }
                self.buffer = vec_spare_capacity_mut(vec);
                self.pos_of_buffer_start += self.pos_within_buf as u64;
                self.pos_within_buf = 0;
                Ok(())
            }
            OutputTarget::Write(..) => self.refresh_buffer(),
        }
    }

    /// Write a byte
    pub fn write_raw_byte(&mut self, byte: u8) -> crate::Result<()> {
        if self.pos_within_buf as usize == self.buffer().len() {
            self.refresh_buffer()?;
        }
        unsafe { (&mut *self.buffer)[self.pos_within_buf as usize].write(byte) };
        self.pos_within_buf += 1;
        Ok(())
    }

    /// Write bytes
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) -> crate::Result<()> {
        if bytes.len() <= self.buffer().len() - self.pos_within_buf {
            let bottom = self.pos_within_buf as usize;
            let top = bottom + (bytes.len() as usize);
            // SAFETY: see the `buffer` field documentation about invariants.
            let buffer = unsafe { &mut (&mut *self.buffer)[bottom..top] };
            maybe_uninit_write_slice(buffer, bytes);
            self.pos_within_buf += bytes.len();
            return Ok(());
        }

        self.refresh_buffer()?;

        assert!(self.pos_within_buf == 0);

        if self.pos_within_buf + bytes.len() < self.buffer().len() {
            // SAFETY: see the `buffer` field documentation about invariants.
            let buffer = unsafe {
                &mut (&mut *self.buffer)[self.pos_within_buf..self.pos_within_buf + bytes.len()]
            };
            maybe_uninit_write_slice(buffer, bytes);
            self.pos_within_buf += bytes.len();
            return Ok(());
        }

        match self.target {
            OutputTarget::Bytes => {
                unreachable!();
            }
            OutputTarget::Write(ref mut write, _) => {
                write.write_all(bytes)?;
            }
            OutputTarget::Vec(ref mut vec) => {
                assert!(self.pos_within_buf == 0);
                vec.extend(bytes);
                self.buffer = vec_spare_capacity_mut(vec);
                self.pos_of_buffer_start += bytes.len() as u64;
            }
        }
        Ok(())
    }

    /// Write a tag
    pub fn write_tag(&mut self, field_number: u32, wire_type: WireType) -> crate::Result<()> {
        self.write_raw_varint32(wire_format::Tag::make(field_number, wire_type).value())
    }

    /// Write varint
    pub fn write_raw_varint32(&mut self, value: u32) -> crate::Result<()> {
        if self.buffer().len() - self.pos_within_buf >= 5 {
            // fast path
            let len = unsafe {
                varint::encode_varint32(value, &mut (&mut *self.buffer)[self.pos_within_buf..])
            };
            self.pos_within_buf += len;
            Ok(())
        } else {
            // slow path
            let buf = &mut [0u8; 5];
            let len = varint::encode_varint32(value, unsafe {
                slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut MaybeUninit<u8>, buf.len())
            });
            self.write_raw_bytes(&buf[..len])
        }
    }

    /// Write varint
    pub fn write_raw_varint64(&mut self, value: u64) -> crate::Result<()> {
        if self.buffer().len() - self.pos_within_buf >= 10 {
            // fast path
            let len = unsafe {
                varint::encode_varint64(value, &mut (&mut *self.buffer)[self.pos_within_buf..])
            };
            self.pos_within_buf += len;
            Ok(())
        } else {
            // slow path
            let buf = &mut [0u8; 10];
            let len = varint::encode_varint64(value, unsafe {
                slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut MaybeUninit<u8>, buf.len())
            });
            self.write_raw_bytes(&buf[..len])
        }
    }

    /// Write 32-bit integer little endian
    pub fn write_raw_little_endian32(&mut self, value: u32) -> crate::Result<()> {
        self.write_raw_bytes(&value.to_le_bytes())
    }

    /// Write 64-bit integer little endian
    pub fn write_raw_little_endian64(&mut self, value: u64) -> crate::Result<()> {
        self.write_raw_bytes(&value.to_le_bytes())
    }

    /// Write `float`
    pub fn write_float_no_tag(&mut self, value: f32) -> crate::Result<()> {
        self.write_raw_little_endian32(value.to_bits())
    }

    /// Write `double`
    pub fn write_double_no_tag(&mut self, value: f64) -> crate::Result<()> {
        self.write_raw_little_endian64(value.to_bits())
    }

    /// Write `float` field
    pub fn write_float(&mut self, field_number: u32, value: f32) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Fixed32)?;
        self.write_float_no_tag(value)?;
        Ok(())
    }

    /// Write `double` field
    pub fn write_double(&mut self, field_number: u32, value: f64) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Fixed64)?;
        self.write_double_no_tag(value)?;
        Ok(())
    }

    /// Write varint
    pub fn write_uint64_no_tag(&mut self, value: u64) -> crate::Result<()> {
        self.write_raw_varint64(value)
    }

    /// Write varint
    pub fn write_uint32_no_tag(&mut self, value: u32) -> crate::Result<()> {
        self.write_raw_varint32(value)
    }

    /// Write varint
    pub fn write_int64_no_tag(&mut self, value: i64) -> crate::Result<()> {
        self.write_raw_varint64(value as u64)
    }

    /// Write varint
    pub fn write_int32_no_tag(&mut self, value: i32) -> crate::Result<()> {
        self.write_raw_varint64(value as u64)
    }

    /// Write zigzag varint
    pub fn write_sint64_no_tag(&mut self, value: i64) -> crate::Result<()> {
        self.write_uint64_no_tag(encode_zig_zag_64(value))
    }

    /// Write zigzag varint
    pub fn write_sint32_no_tag(&mut self, value: i32) -> crate::Result<()> {
        self.write_uint32_no_tag(encode_zig_zag_32(value))
    }

    /// Write `fixed64`
    pub fn write_fixed64_no_tag(&mut self, value: u64) -> crate::Result<()> {
        self.write_raw_little_endian64(value)
    }

    /// Write `fixed32`
    pub fn write_fixed32_no_tag(&mut self, value: u32) -> crate::Result<()> {
        self.write_raw_little_endian32(value)
    }

    /// Write `sfixed64`
    pub fn write_sfixed64_no_tag(&mut self, value: i64) -> crate::Result<()> {
        self.write_raw_little_endian64(value as u64)
    }

    /// Write `sfixed32`
    pub fn write_sfixed32_no_tag(&mut self, value: i32) -> crate::Result<()> {
        self.write_raw_little_endian32(value as u32)
    }

    /// Write `bool`
    pub fn write_bool_no_tag(&mut self, value: bool) -> crate::Result<()> {
        self.write_raw_varint32(if value { 1 } else { 0 })
    }

    /// Write `enum`
    pub fn write_enum_no_tag(&mut self, value: i32) -> crate::Result<()> {
        self.write_int32_no_tag(value)
    }

    /// Write `enum`
    pub fn write_enum_obj_no_tag<E>(&mut self, value: E) -> crate::Result<()>
    where
        E: Enum,
    {
        self.write_enum_no_tag(value.value())
    }

    /// Write `enum`
    pub fn write_enum_or_unknown_no_tag<E>(&mut self, value: EnumOrUnknown<E>) -> crate::Result<()>
    where
        E: Enum,
    {
        self.write_enum_no_tag(value.value())
    }

    /// Write unknown value
    pub fn write_unknown_no_tag(&mut self, unknown: UnknownValueRef) -> crate::Result<()> {
        match unknown {
            UnknownValueRef::Fixed64(fixed64) => self.write_raw_little_endian64(fixed64),
            UnknownValueRef::Fixed32(fixed32) => self.write_raw_little_endian32(fixed32),
            UnknownValueRef::Varint(varint) => self.write_raw_varint64(varint),
            UnknownValueRef::LengthDelimited(bytes) => self.write_bytes_no_tag(bytes),
        }
    }

    /// Write `uint64` field
    pub fn write_uint64(&mut self, field_number: u32, value: u64) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Varint)?;
        self.write_uint64_no_tag(value)?;
        Ok(())
    }

    /// Write `uint32` field
    pub fn write_uint32(&mut self, field_number: u32, value: u32) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Varint)?;
        self.write_uint32_no_tag(value)?;
        Ok(())
    }

    /// Write `int64` field
    pub fn write_int64(&mut self, field_number: u32, value: i64) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Varint)?;
        self.write_int64_no_tag(value)?;
        Ok(())
    }

    /// Write `int32` field
    pub fn write_int32(&mut self, field_number: u32, value: i32) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Varint)?;
        self.write_int32_no_tag(value)?;
        Ok(())
    }

    /// Write `sint64` field
    pub fn write_sint64(&mut self, field_number: u32, value: i64) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Varint)?;
        self.write_sint64_no_tag(value)?;
        Ok(())
    }

    /// Write `sint32` field
    pub fn write_sint32(&mut self, field_number: u32, value: i32) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Varint)?;
        self.write_sint32_no_tag(value)?;
        Ok(())
    }

    /// Write `fixed64` field
    pub fn write_fixed64(&mut self, field_number: u32, value: u64) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Fixed64)?;
        self.write_fixed64_no_tag(value)?;
        Ok(())
    }

    /// Write `fixed32` field
    pub fn write_fixed32(&mut self, field_number: u32, value: u32) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Fixed32)?;
        self.write_fixed32_no_tag(value)?;
        Ok(())
    }

    /// Write `sfixed64` field
    pub fn write_sfixed64(&mut self, field_number: u32, value: i64) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Fixed64)?;
        self.write_sfixed64_no_tag(value)?;
        Ok(())
    }

    /// Write `sfixed32` field
    pub fn write_sfixed32(&mut self, field_number: u32, value: i32) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Fixed32)?;
        self.write_sfixed32_no_tag(value)?;
        Ok(())
    }

    /// Write `bool` field
    pub fn write_bool(&mut self, field_number: u32, value: bool) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Varint)?;
        self.write_bool_no_tag(value)?;
        Ok(())
    }

    /// Write `enum` field
    pub fn write_enum(&mut self, field_number: u32, value: i32) -> crate::Result<()> {
        self.write_tag(field_number, WireType::Varint)?;
        self.write_enum_no_tag(value)?;
        Ok(())
    }

    /// Write `enum` field
    pub fn write_enum_obj<E>(&mut self, field_number: u32, value: E) -> crate::Result<()>
    where
        E: Enum,
    {
        self.write_enum(field_number, value.value())
    }

    /// Write `enum` field
    pub fn write_enum_or_unknown<E>(
        &mut self,
        field_number: u32,
        value: EnumOrUnknown<E>,
    ) -> crate::Result<()>
    where
        E: Enum,
    {
        self.write_enum(field_number, value.value())
    }

    /// Write unknown field
    pub fn write_unknown(
        &mut self,
        field_number: u32,
        value: UnknownValueRef,
    ) -> crate::Result<()> {
        self.write_tag(field_number, value.wire_type())?;
        self.write_unknown_no_tag(value)?;
        Ok(())
    }

    /// Write unknown fields
    pub fn write_unknown_fields(&mut self, fields: &UnknownFields) -> crate::Result<()> {
        for (number, values) in fields {
            for value in values {
                self.write_unknown(number, value)?;
            }
        }
        Ok(())
    }

    /// Write unknown fields sorting them by name
    pub(crate) fn write_unknown_fields_sorted(
        &mut self,
        fields: &UnknownFields,
    ) -> crate::Result<()> {
        let mut fields: Vec<_> = fields.iter().collect();
        fields.sort_by_key(|(n, _)| *n);
        for (number, values) in fields {
            for value in values {
                self.write_unknown(number, value)?;
            }
        }
        Ok(())
    }

    /// Write bytes
    pub fn write_bytes_no_tag(&mut self, bytes: &[u8]) -> crate::Result<()> {
        self.write_raw_varint32(bytes.len() as u32)?;
        self.write_raw_bytes(bytes)?;
        Ok(())
    }

    /// Write string
    pub fn write_string_no_tag(&mut self, s: &str) -> crate::Result<()> {
        self.write_bytes_no_tag(s.as_bytes())
    }

    /// Write message
    pub fn write_message_no_tag<M: Message>(&mut self, msg: &M) -> crate::Result<()> {
        msg.write_length_delimited_to(self)
    }

    /// Write dynamic message
    pub fn write_message_no_tag_dyn(&mut self, msg: &dyn MessageDyn) -> crate::Result<()> {
        let size = msg.compute_size_dyn();
        let size = check_message_size(size)?;
        self.write_raw_varint32(size)?;
        msg.write_to_dyn(self)?;
        Ok(())
    }

    /// Write `bytes` field
    pub fn write_bytes(&mut self, field_number: u32, bytes: &[u8]) -> crate::Result<()> {
        self.write_tag(field_number, WireType::LengthDelimited)?;
        self.write_bytes_no_tag(bytes)?;
        Ok(())
    }

    /// Write `string` field
    pub fn write_string(&mut self, field_number: u32, s: &str) -> crate::Result<()> {
        self.write_tag(field_number, WireType::LengthDelimited)?;
        self.write_string_no_tag(s)?;
        Ok(())
    }

    /// Write repeated packed float values.
    pub fn write_repeated_packed_float_no_tag(&mut self, values: &[f32]) -> crate::Result<()> {
        for v in values {
            self.write_float_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed float.
    pub fn write_repeated_packed_float(
        &mut self,
        field_number: u32,
        values: &[f32],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_fixed_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_float_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed double values.
    pub fn write_repeated_packed_double_no_tag(&mut self, values: &[f64]) -> crate::Result<()> {
        for v in values {
            self.write_double_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed double.
    pub fn write_repeated_packed_double(
        &mut self,
        field_number: u32,
        values: &[f64],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_fixed_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_double_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed fixed32 values.
    pub fn write_repeated_packed_fixed32_no_tag(&mut self, values: &[u32]) -> crate::Result<()> {
        // TODO: these can be memcopied.
        for v in values {
            self.write_fixed32_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed fixed32.
    pub fn write_repeated_packed_fixed32(
        &mut self,
        field_number: u32,
        values: &[u32],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_fixed_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_fixed32_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed fixed64 values.
    pub fn write_repeated_packed_fixed64_no_tag(&mut self, values: &[u64]) -> crate::Result<()> {
        for v in values {
            self.write_fixed64_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed fixed64.
    pub fn write_repeated_packed_fixed64(
        &mut self,
        field_number: u32,
        values: &[u64],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_fixed_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_fixed64_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed sfixed32 values.
    pub fn write_repeated_packed_sfixed32_no_tag(&mut self, values: &[i32]) -> crate::Result<()> {
        for v in values {
            self.write_sfixed32_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed sfixed32.
    pub fn write_repeated_packed_sfixed32(
        &mut self,
        field_number: u32,
        values: &[i32],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_fixed_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_sfixed32_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed sfixed64 values.
    pub fn write_repeated_packed_sfixed64_no_tag(&mut self, values: &[i64]) -> crate::Result<()> {
        for v in values {
            self.write_sfixed64_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed sfixed64.
    pub fn write_repeated_packed_sfixed64(
        &mut self,
        field_number: u32,
        values: &[i64],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_fixed_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_sfixed64_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed int32 values.
    pub fn write_repeated_packed_int32_no_tag(&mut self, values: &[i32]) -> crate::Result<()> {
        for v in values {
            self.write_int32_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed int32.
    pub fn write_repeated_packed_int32(
        &mut self,
        field_number: u32,
        values: &[i32],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_varint_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_int32_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed int64 values.
    pub fn write_repeated_packed_int64_no_tag(&mut self, values: &[i64]) -> crate::Result<()> {
        for v in values {
            self.write_int64_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed int64.
    pub fn write_repeated_packed_int64(
        &mut self,
        field_number: u32,
        values: &[i64],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_varint_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_int64_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed uint32 values.
    pub fn write_repeated_packed_uint32_no_tag(&mut self, values: &[u32]) -> crate::Result<()> {
        for v in values {
            self.write_uint32_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed uint32.
    pub fn write_repeated_packed_uint32(
        &mut self,
        field_number: u32,
        values: &[u32],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_varint_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_uint32_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed uint64 values.
    pub fn write_repeated_packed_uint64_no_tag(&mut self, values: &[u64]) -> crate::Result<()> {
        for v in values {
            self.write_uint64_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed uint64.
    pub fn write_repeated_packed_uint64(
        &mut self,
        field_number: u32,
        values: &[u64],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_varint_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_uint64_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed sint32 values.
    pub fn write_repeated_packed_sint32_no_tag(&mut self, values: &[i32]) -> crate::Result<()> {
        for v in values {
            self.write_sint32_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed sint32.
    pub fn write_repeated_packed_sint32(
        &mut self,
        field_number: u32,
        values: &[i32],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_varint_zigzag_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_sint32_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed sint64 values.
    pub fn write_repeated_packed_sint64_no_tag(&mut self, values: &[i64]) -> crate::Result<()> {
        for v in values {
            self.write_sint64_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed sint64.
    pub fn write_repeated_packed_sint64(
        &mut self,
        field_number: u32,
        values: &[i64],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_varint_zigzag_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_sint64_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed bool values.
    pub fn write_repeated_packed_bool_no_tag(&mut self, values: &[bool]) -> crate::Result<()> {
        for v in values {
            self.write_bool_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed bool.
    pub fn write_repeated_packed_bool(
        &mut self,
        field_number: u32,
        values: &[bool],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_fixed_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_bool_no_tag(values)?;
        Ok(())
    }

    /// Write repeated packed enum values.
    pub fn write_repeated_packed_enum_no_tag(&mut self, values: &[i32]) -> crate::Result<()> {
        for v in values {
            self.write_enum_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write repeated packed enum values.
    pub fn write_repeated_packed_enum_or_unknown_no_tag<E: Enum>(
        &mut self,
        values: &[EnumOrUnknown<E>],
    ) -> crate::Result<()> {
        for v in values {
            self.write_enum_or_unknown_no_tag(*v)?;
        }
        Ok(())
    }

    /// Write field header and data for repeated packed enum.
    pub fn write_repeated_packed_enum_or_unknown<E: Enum>(
        &mut self,
        field_number: u32,
        values: &[EnumOrUnknown<E>],
    ) -> crate::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_tag(field_number, WireType::LengthDelimited)?;
        let data_size = vec_packed_enum_or_unknown_data_size(values);
        self.write_raw_varint32(data_size as u32)?;
        self.write_repeated_packed_enum_or_unknown_no_tag(values)?;
        Ok(())
    }

    /// Write `message` field
    pub fn write_message<M: MessageFull>(
        &mut self,
        field_number: u32,
        msg: &M,
    ) -> crate::Result<()> {
        self.write_tag(field_number, WireType::LengthDelimited)?;
        self.write_message_no_tag(msg)?;
        Ok(())
    }

    /// Write dynamic `message` field
    pub fn write_message_dyn(
        &mut self,
        field_number: u32,
        msg: &dyn MessageDyn,
    ) -> crate::Result<()> {
        self.write_tag(field_number, WireType::LengthDelimited)?;
        self.write_message_no_tag_dyn(msg)?;
        Ok(())
    }
}

impl<'a> Write for CodedOutputStream<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_raw_bytes(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        CodedOutputStream::flush(self).map_err(Into::into)
    }
}

impl<'a> Drop for CodedOutputStream<'a> {
    fn drop(&mut self) {
        // This may panic
        CodedOutputStream::flush(self).expect("failed to flush");
    }
}

#[cfg(test)]
mod test {
    use std::iter;

    use super::*;
    use crate::hex::decode_hex;
    use crate::hex::encode_hex;

    fn test_write<F>(expected: &str, mut gen: F)
    where
        F: FnMut(&mut CodedOutputStream) -> crate::Result<()>,
    {
        let expected_bytes = decode_hex(expected);

        // write to Write
        {
            let mut v = Vec::new();
            {
                let mut os = CodedOutputStream::new(&mut v as &mut dyn Write);
                gen(&mut os).unwrap();
                os.flush().unwrap();
            }
            assert_eq!(encode_hex(&expected_bytes), encode_hex(&v));
        }

        // write to &[u8]
        {
            let mut r = Vec::with_capacity(expected_bytes.len());
            r.resize(expected_bytes.len(), 0);
            {
                let mut os = CodedOutputStream::bytes(&mut r);
                gen(&mut os).unwrap();
                os.check_eof();
            }
            assert_eq!(encode_hex(&expected_bytes), encode_hex(&r));
        }

        // write to Vec<u8>
        {
            let mut r = Vec::new();
            r.extend(&[11, 22, 33, 44, 55, 66, 77]);
            {
                let mut os = CodedOutputStream::vec(&mut r);
                gen(&mut os).unwrap();
                os.flush().unwrap();
            }

            r.drain(..7);
            assert_eq!(encode_hex(&expected_bytes), encode_hex(&r));
        }
    }

    #[test]
    fn test_output_stream_write_raw_byte() {
        test_write("a1", |os| os.write_raw_byte(0xa1));
    }

    #[test]
    fn test_output_stream_write_tag() {
        test_write("08", |os| os.write_tag(1, WireType::Varint));
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow for this test.
    fn test_output_stream_write_raw_bytes() {
        test_write("00 ab", |os| os.write_raw_bytes(&[0x00, 0xab]));

        let expected = iter::repeat("01 02 03 04")
            .take(2048)
            .collect::<Vec<_>>()
            .join(" ");
        test_write(&expected, |os| {
            for _ in 0..2048 {
                os.write_raw_bytes(&[0x01, 0x02, 0x03, 0x04])?;
            }

            Ok(())
        });
    }

    #[test]
    fn test_output_stream_write_raw_varint32() {
        test_write("96 01", |os| os.write_raw_varint32(150));
        test_write("ff ff ff ff 0f", |os| os.write_raw_varint32(0xffffffff));
    }

    #[test]
    fn test_output_stream_write_raw_varint64() {
        test_write("96 01", |os| os.write_raw_varint64(150));
        test_write("ff ff ff ff ff ff ff ff ff 01", |os| {
            os.write_raw_varint64(0xffffffffffffffff)
        });
    }

    #[test]
    fn test_output_stream_write_int32_no_tag() {
        test_write("ff ff ff ff ff ff ff ff ff 01", |os| {
            os.write_int32_no_tag(-1)
        });
    }

    #[test]
    fn test_output_stream_write_int64_no_tag() {
        test_write("ff ff ff ff ff ff ff ff ff 01", |os| {
            os.write_int64_no_tag(-1)
        });
    }

    #[test]
    fn test_output_stream_write_raw_little_endian32() {
        test_write("f1 e2 d3 c4", |os| os.write_raw_little_endian32(0xc4d3e2f1));
    }

    #[test]
    fn test_output_stream_write_float_no_tag() {
        test_write("95 73 13 61", |os| os.write_float_no_tag(17e19));
    }

    #[test]
    fn test_output_stream_write_double_no_tag() {
        test_write("40 d5 ab 68 b3 07 3d 46", |os| {
            os.write_double_no_tag(23e29)
        });
    }

    #[test]
    fn test_output_stream_write_raw_little_endian64() {
        test_write("f1 e2 d3 c4 b5 a6 07 f8", |os| {
            os.write_raw_little_endian64(0xf807a6b5c4d3e2f1)
        });
    }

    #[test]
    fn test_output_stream_io_write() {
        let expected = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];

        // write to Write
        {
            let mut v = Vec::new();
            {
                let mut os = CodedOutputStream::new(&mut v as &mut dyn Write);
                Write::write(&mut os, &expected).expect("io::Write::write");
                Write::flush(&mut os).expect("io::Write::flush");
            }
            assert_eq!(expected, *v);
        }

        // write to &[u8]
        {
            let mut v = Vec::with_capacity(expected.len());
            v.resize(expected.len(), 0);
            {
                let mut os = CodedOutputStream::bytes(&mut v);
                Write::write(&mut os, &expected).expect("io::Write::write");
                Write::flush(&mut os).expect("io::Write::flush");
                os.check_eof();
            }
            assert_eq!(expected, *v);
        }

        // write to Vec<u8>
        {
            let mut v = Vec::new();
            {
                let mut os = CodedOutputStream::vec(&mut v);
                Write::write(&mut os, &expected).expect("io::Write::write");
                Write::flush(&mut os).expect("io::Write::flush");
            }
            assert_eq!(expected, *v);
        }
    }

    #[test]
    fn flush_for_vec_does_not_allocate_more() {
        let mut v = Vec::with_capacity(10);
        {
            let mut os = CodedOutputStream::vec(&mut v);
            for i in 0..10 {
                os.write_raw_byte(i as u8).unwrap();
            }
            os.flush().unwrap();
        }
        assert_eq!(10, v.len());
        // Previously, this allocated more data in buf.
        assert_eq!(10, v.capacity());
    }

    #[test]
    fn total_bytes_written_to_bytes() {
        let mut buf = vec![0; 10];
        let mut stream = CodedOutputStream::bytes(&mut buf);
        assert_eq!(0, stream.total_bytes_written());
        stream.write_raw_bytes(&[11, 22]).unwrap();
        assert_eq!(2, stream.total_bytes_written());
        stream.write_raw_bytes(&[33, 44, 55]).unwrap();
        assert_eq!(5, stream.total_bytes_written());
    }

    #[test]
    fn total_bytes_written_to_vec() {
        let mut buf = Vec::new();
        let mut stream = CodedOutputStream::vec(&mut buf);
        for i in 0..100 {
            stream.write_raw_bytes(&[0, 1, 2]).unwrap();
            assert_eq!((i + 1) * 3, stream.total_bytes_written());
        }
    }
}
