use crate::property::ArrayProp;
use crate::property::{Prop, PropertySelector, RawProperty};
use core_foundation::string::CFString;
use coreaudio_sys::kAudioObjectPropertyBaseClass;
use coreaudio_sys::kAudioObjectPropertyClass;
use coreaudio_sys::kAudioObjectPropertyName;
use coreaudio_sys::kAudioObjectPropertyOwnedObjects;
use coreaudio_sys::kAudioObjectPropertyOwner;
use coreaudio_sys::AudioClassID;
use coreaudio_sys::AudioObjectID;

pub trait AudioObject {
    fn get_property(&self, sel: PropertySelector) -> &dyn RawProperty;
    fn get_property_mut(&mut self, sel: PropertySelector) -> &mut dyn RawProperty;
}
#[derive(Debug)]
pub struct AudioObjectBase {
    pub base_class: Prop<AudioClassID, kAudioObjectPropertyBaseClass, false>,
    pub class: Prop<AudioClassID, kAudioObjectPropertyClass, false>,
    pub owner: Prop<AudioObjectID, kAudioObjectPropertyOwner, false>,
    pub owned_objects: ArrayProp<AudioObjectID, kAudioObjectPropertyOwnedObjects, false>,
    pub name: Prop<CFString, kAudioObjectPropertyName, false>,
}
impl AudioObjectBase {
    pub fn new(
        base_class: AudioClassID,
        class: AudioClassID,
        owner: AudioObjectID,
        name: &'static str,
    ) -> Self {
        Self {
            base_class: Prop(base_class),
            class: Prop(class),
            owner: Prop(owner),
            owned_objects: ArrayProp::new(),
            name: Prop(CFString::from_static_string(name)),
        }
    }
}
