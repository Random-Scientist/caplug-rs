use cahal::{
    core_foundation::base::CFAllocatorRef,
    plugin_driver_interface::AudioServerPluginDriverInterface,
};

pub struct Plugin {}
impl AudioServerPluginDriverInterface for Plugin {
    const NAME: &'static str = "test_plugin";

    fn new(cf_allocator: CFAllocatorRef) -> Self {
        todo!()
    }

    fn init(&self, host: &AudioServerPlugInHostInterface) -> cahal::OSStatus {
        todo!()
    }
}
