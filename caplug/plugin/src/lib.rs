use cahal::{
    core_foundation::base::CFAllocatorRef, entry_point,
    plugin_driver_interface::AudioServerPluginDriverInterface,
    raw_plugin_driver_interface::PluginHostInterface,
};

pub struct TestPlugin {
    _value: u8,
}
impl AudioServerPluginDriverInterface for TestPlugin {
    type DeviceConfigurationChangeInfo = ();
    const NAME: &'static str = "test_plugin";

    fn create(_cf_allocator: CFAllocatorRef) -> Self {
        Self { _value: 0 }
    }

    fn init(&self, _host: PluginHostInterface<Self>) -> cahal::os_err::OSStatus {
        Ok(())
    }
}

entry_point!(TestPlugin);
