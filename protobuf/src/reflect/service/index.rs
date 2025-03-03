use crate::descriptor::MethodDescriptorProto;
use crate::descriptor::ServiceDescriptorProto;
use crate::reflect::field::index::ForwardProtobufTypeBox;
use crate::reflect::file::building::FileDescriptorBuilding;

#[derive(Debug)]
pub(crate) struct ServiceIndex {
    pub(crate) methods: Vec<MethodIndex>,
}

impl ServiceIndex {
    pub(crate) fn index(
        proto: &ServiceDescriptorProto,
        building: &FileDescriptorBuilding,
    ) -> ServiceIndex {
        ServiceIndex {
            methods: proto
                .method
                .iter()
                .map(|method| MethodIndex::index(method, building))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct MethodIndex {
    pub(crate) input_type: ForwardProtobufTypeBox,
    pub(crate) output_type: ForwardProtobufTypeBox,
}

impl MethodIndex {
    pub(crate) fn index(
        proto: &MethodDescriptorProto,
        building: &FileDescriptorBuilding,
    ) -> MethodIndex {
        let input_type = building.resolve_message(proto.input_type());
        let output_type = building.resolve_message(proto.output_type());
        MethodIndex {
            input_type,
            output_type,
        }
    }
}
