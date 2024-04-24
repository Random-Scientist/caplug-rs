use cahal::{
    property::Prop,
    raw::{kAudioObjectPropertyBaseClass, kAudioObjectPropertyClass, AudioClassID},
};

pub struct Plugin {}

pub struct TestPluginBox {
    base_class: Prop<AudioClassID, kAudioObjectPropertyBaseClass, false>,
    class: Prop<AudioClassID, kAudioObjectPropertyClass, false>,
}
