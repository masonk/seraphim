// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct Example {
    // message fields
    pub features: ::protobuf::SingularPtrField<super::feature::Features>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Example {}

impl Example {
    pub fn new() -> Example {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Example {
        static mut instance: ::protobuf::lazy::Lazy<Example> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Example,
        };
        unsafe {
            instance.get(Example::new)
        }
    }

    // .tensorflow.Features features = 1;

    pub fn clear_features(&mut self) {
        self.features.clear();
    }

    pub fn has_features(&self) -> bool {
        self.features.is_some()
    }

    // Param is passed by value, moved
    pub fn set_features(&mut self, v: super::feature::Features) {
        self.features = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_features(&mut self) -> &mut super::feature::Features {
        if self.features.is_none() {
            self.features.set_default();
        }
        self.features.as_mut().unwrap()
    }

    // Take field
    pub fn take_features(&mut self) -> super::feature::Features {
        self.features.take().unwrap_or_else(|| super::feature::Features::new())
    }

    pub fn get_features(&self) -> &super::feature::Features {
        self.features.as_ref().unwrap_or_else(|| super::feature::Features::default_instance())
    }

    fn get_features_for_reflect(&self) -> &::protobuf::SingularPtrField<super::feature::Features> {
        &self.features
    }

    fn mut_features_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::feature::Features> {
        &mut self.features
    }
}

impl ::protobuf::Message for Example {
    fn is_initialized(&self) -> bool {
        for v in &self.features {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.features)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.features.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.features.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Example {
    fn new() -> Example {
        Example::new()
    }

    fn descriptor_static(_: ::std::option::Option<Example>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::feature::Features>>(
                    "features",
                    Example::get_features_for_reflect,
                    Example::mut_features_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Example>(
                    "Example",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Example {
    fn clear(&mut self) {
        self.clear_features();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Example {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Example {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SequenceExample {
    // message fields
    pub context: ::protobuf::SingularPtrField<super::feature::Features>,
    pub feature_lists: ::protobuf::SingularPtrField<super::feature::FeatureLists>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for SequenceExample {}

impl SequenceExample {
    pub fn new() -> SequenceExample {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static SequenceExample {
        static mut instance: ::protobuf::lazy::Lazy<SequenceExample> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const SequenceExample,
        };
        unsafe {
            instance.get(SequenceExample::new)
        }
    }

    // .tensorflow.Features context = 1;

    pub fn clear_context(&mut self) {
        self.context.clear();
    }

    pub fn has_context(&self) -> bool {
        self.context.is_some()
    }

    // Param is passed by value, moved
    pub fn set_context(&mut self, v: super::feature::Features) {
        self.context = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_context(&mut self) -> &mut super::feature::Features {
        if self.context.is_none() {
            self.context.set_default();
        }
        self.context.as_mut().unwrap()
    }

    // Take field
    pub fn take_context(&mut self) -> super::feature::Features {
        self.context.take().unwrap_or_else(|| super::feature::Features::new())
    }

    pub fn get_context(&self) -> &super::feature::Features {
        self.context.as_ref().unwrap_or_else(|| super::feature::Features::default_instance())
    }

    fn get_context_for_reflect(&self) -> &::protobuf::SingularPtrField<super::feature::Features> {
        &self.context
    }

    fn mut_context_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::feature::Features> {
        &mut self.context
    }

    // .tensorflow.FeatureLists feature_lists = 2;

    pub fn clear_feature_lists(&mut self) {
        self.feature_lists.clear();
    }

    pub fn has_feature_lists(&self) -> bool {
        self.feature_lists.is_some()
    }

    // Param is passed by value, moved
    pub fn set_feature_lists(&mut self, v: super::feature::FeatureLists) {
        self.feature_lists = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_feature_lists(&mut self) -> &mut super::feature::FeatureLists {
        if self.feature_lists.is_none() {
            self.feature_lists.set_default();
        }
        self.feature_lists.as_mut().unwrap()
    }

    // Take field
    pub fn take_feature_lists(&mut self) -> super::feature::FeatureLists {
        self.feature_lists.take().unwrap_or_else(|| super::feature::FeatureLists::new())
    }

    pub fn get_feature_lists(&self) -> &super::feature::FeatureLists {
        self.feature_lists.as_ref().unwrap_or_else(|| super::feature::FeatureLists::default_instance())
    }

    fn get_feature_lists_for_reflect(&self) -> &::protobuf::SingularPtrField<super::feature::FeatureLists> {
        &self.feature_lists
    }

    fn mut_feature_lists_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<super::feature::FeatureLists> {
        &mut self.feature_lists
    }
}

impl ::protobuf::Message for SequenceExample {
    fn is_initialized(&self) -> bool {
        for v in &self.context {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.feature_lists {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.context)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.feature_lists)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.context.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.feature_lists.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.context.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.feature_lists.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for SequenceExample {
    fn new() -> SequenceExample {
        SequenceExample::new()
    }

    fn descriptor_static(_: ::std::option::Option<SequenceExample>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::feature::Features>>(
                    "context",
                    SequenceExample::get_context_for_reflect,
                    SequenceExample::mut_context_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::feature::FeatureLists>>(
                    "feature_lists",
                    SequenceExample::get_feature_lists_for_reflect,
                    SequenceExample::mut_feature_lists_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<SequenceExample>(
                    "SequenceExample",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for SequenceExample {
    fn clear(&mut self) {
        self.clear_context();
        self.clear_feature_lists();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SequenceExample {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SequenceExample {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n%tensorflow/core/example/example.proto\x12\ntensorflow\x1a%tensorflow/\
    core/example/feature.proto\";\n\x07Example\x120\n\x08features\x18\x01\
    \x20\x01(\x0b2\x14.tensorflow.FeaturesR\x08features\"\x80\x01\n\x0fSeque\
    nceExample\x12.\n\x07context\x18\x01\x20\x01(\x0b2\x14.tensorflow.Featur\
    esR\x07context\x12=\n\rfeature_lists\x18\x02\x20\x01(\x0b2\x18.tensorflo\
    w.FeatureListsR\x0cfeatureListsB,\n\x16org.tensorflow.exampleB\rExampleP\
    rotosP\x01\xf8\x01\x01b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
