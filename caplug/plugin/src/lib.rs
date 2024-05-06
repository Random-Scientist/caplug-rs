use cahal::{
    core_foundation::base::CFAllocatorRef, entry_point,
    plugin_driver_interface::AudioServerPluginDriverInterface,
    raw_plugin_driver_interface::PluginHostInterface,
};

pub struct TestPlugin {
    value: u8,
}
impl AudioServerPluginDriverInterface for TestPlugin {
    const NAME: &'static str = "test_plugin";

    fn create(_cf_allocator: CFAllocatorRef) -> Self {
        Self { value: 0 }
    }

    fn init(&self, _host: PluginHostInterface) -> cahal::os_err::OSStatus {
        Ok(())
    }
}

entry_point!(TestPlugin);
