use ash::vk;
struct Data<const SIZE: usize> {
    data: [std::ffi::c_void; SIZE],
}
pub struct PushConstant<const SIZE: usize> {
    range: vk::PushConstantRange,
    data: [std::ffi::c_void; SIZE],
}
