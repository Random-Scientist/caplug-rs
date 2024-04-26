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

pub trait AudioObject: HasProperties {
    fn subobjects(&self) -> &[&dyn AudioObject];
    fn subobjects_mut(&mut self) -> &mut [&mut dyn AudioObject];

    fn get_property(&self, sel: PropertySelector) -> Option<&dyn RawProperty> {
        if let Some(prop) = self.get_object_property(sel) {
            return Some(prop);
        }
        for obj in self.subobjects() {
            if let Some(prop) = obj.get_property(sel) {
                return Some(prop);
            }
        }
        None
    }
    fn get_property_mut(&mut self, sel: PropertySelector) -> Option<&mut dyn RawProperty> {
        if let Some(prop) = self.get_object_property_mut(sel) {
            return Some(prop);
        }
        for obj in self.subobjects_mut() {
            let prop = obj.get_property_mut(sel);
            if prop.is_some() {
                return prop;
            }
        }
        None
    }
}
fn test(r: &mut dyn AudioObject, sel: PropertySelector) -> Option<&mut dyn RawProperty> {
    for obj in r.subobjects_mut() {
        let prop = obj.get_property_mut(sel);
        if prop.is_some() {
            return prop;
        }
    }
    None
}
pub trait HasProperties {
    fn get_object_property(&self, sel: PropertySelector) -> Option<&dyn RawProperty>;
    fn get_object_property_mut(&mut self, sel: PropertySelector) -> Option<&mut dyn RawProperty>;
}

#[derive(Debug)]
pub struct AudioObjectBase {
    pub base_class: Prop<AudioClassID, kAudioObjectPropertyBaseClass, false>,
    pub class: Prop<AudioClassID, kAudioObjectPropertyClass, false>,
    pub owner: Prop<AudioObjectID, kAudioObjectPropertyOwner, false>,
    pub owned_objects: ArrayProp<AudioObjectID, kAudioObjectPropertyOwnedObjects, false>,
    pub name: Prop<CFString, kAudioObjectPropertyName, false>,
}
impl HasProperties for AudioObjectBase {
    fn get_object_property(&self, sel: PropertySelector) -> Option<&dyn RawProperty> {
        Some(match sel.into() {
            kAudioObjectPropertyBaseClass => &self.base_class,
            kAudioObjectPropertyClass => &self.class,
            kAudioObjectPropertyName => &self.name,
            kAudioObjectPropertyOwnedObjects => &self.owned_objects,
            kAudioObjectPropertyOwner => &self.owner,
            _ => return None,
        })
    }

    fn get_object_property_mut(&mut self, sel: PropertySelector) -> Option<&mut dyn RawProperty> {
        Some(match sel.into() {
            kAudioObjectPropertyBaseClass => &mut self.base_class,
            kAudioObjectPropertyClass => &mut self.class,
            kAudioObjectPropertyName => &mut self.name,
            kAudioObjectPropertyOwnedObjects => &mut self.owned_objects,
            kAudioObjectPropertyOwner => &mut self.owner,
            _ => return None,
        })
    }
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
