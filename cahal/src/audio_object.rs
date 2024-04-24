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
pub struct AudioObjectBase {
    base_class: Prop<AudioClassID, kAudioObjectPropertyBaseClass, false>,
    class: Prop<AudioClassID, kAudioObjectPropertyClass, false>,
    owner: Prop<AudioObjectID, kAudioObjectPropertyOwner, false>,
    owned_objects: ArrayProp<AudioObjectID, kAudioObjectPropertyOwnedObjects, false>,

    name: Option<Prop<CFString, kAudioObjectPropertyName, false>>,
}
