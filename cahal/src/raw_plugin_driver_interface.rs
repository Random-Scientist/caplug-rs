use std::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use core_foundation::{propertylist::CFPropertyListRef, string::CFStringRef};
use coreaudio_sys::{
    pid_t, AudioObjectID, AudioObjectPropertyAddress, AudioServerPlugInClientInfo,
    AudioServerPlugInDriverInterface, AudioServerPlugInDriverRef, AudioServerPlugInHostInterface,
    AudioServerPlugInHostRef, AudioServerPlugInIOCycleInfo, CFAllocatorRef, CFDictionaryRef,
    OSStatus, HRESULT, LPVOID, REFIID, ULONG,
};

use crate::os_err::{result_from_raw, OSStatusError, ResultExt};

pub trait RawAudioServerPlugInDriverInterface {
    /// Holds the full implementation of this trait in a struct of function pointers
    const IMPLEMENTATION: AudioServerPlugInDriverInterface = AudioServerPlugInDriverInterface {
        _reserved: ptr::null_mut(),
        QueryInterface: Some(Self::query_interface),
        AddRef: Some(Self::retain),
        Release: Some(Self::release),
        Initialize: Some(Self::initialize),
        CreateDevice: Some(Self::create_device),
        DestroyDevice: Some(Self::destroy_device),
        AddDeviceClient: Some(Self::add_device_client),
        RemoveDeviceClient: Some(Self::remove_device_client),
        PerformDeviceConfigurationChange: Some(Self::perform_device_configuration_change),
        AbortDeviceConfigurationChange: Some(Self::abort_device_configuration_change),
        HasProperty: Some(Self::has_property),
        IsPropertySettable: Some(Self::is_property_settable),
        GetPropertyDataSize: Some(Self::get_property_data_size),
        GetPropertyData: Some(Self::get_property_data),
        SetPropertyData: Some(Self::set_property_data),
        StartIO: Some(Self::start_io),
        StopIO: Some(Self::stop_io),
        GetZeroTimeStamp: Some(Self::get_zero_time_stamp),
        WillDoIOOperation: Some(Self::will_do_io_operation),
        BeginIOOperation: Some(Self::begin_io_operation),
        DoIOOperation: Some(Self::do_io_operation),
        EndIOOperation: Some(Self::end_io_operation),
    };
    ///	This is the CFPlugIn factory function. Its job is to create the implementation for the given
    ///	type provided that the type is supported. Because this driver is simple and all its
    ///	initialization is handled via static iniitalization when the bundle is loaded, all that
    ///	needs to be done is to return the AudioServerPlugInDriverRef that points to the driver's
    ///	interface. A more complicated driver would create any base line objects it needs to satisfy
    ///	the IUnknown methods that are used to discover that actual interface to talk to the driver.
    ///	The majority of the driver's initilization should be handled in the Initialize() method of
    ///	the driver's AudioServerPlugInDriverInterface.
    unsafe extern "C" fn create(
        alloc: CFAllocatorRef,
        requested_uuid: crate::base::CFUUIDRef,
    ) -> *mut c_void;

    ///	This function is called by the HAL to get the interface to talk to the plug-in through.
    ///	AudioServerPlugIns are required to support the IUnknown interface and the
    ///	AudioServerPlugInDriverInterface. As it happens, all interfaces must also provide the
    ///	IUnknown interface, so we can always just return the single interface we made with
    ///	gAudioServerPlugInDriverInterfacePtr regardless of which one is asked for.
    unsafe extern "C" fn query_interface(
        driver: *mut c_void,
        in_uuid: REFIID,
        out_interface: *mut LPVOID,
    ) -> HRESULT;

    unsafe extern "C" fn retain(driver: *mut c_void) -> ULONG;
    unsafe extern "C" fn release(driver: *mut c_void) -> ULONG;

    ///	The job of this method is, as the name implies, to get the driver initialized. One specific
    ///	thing that needs to be done is to store the AudioServerPlugInHostRef so that it can be used
    ///	later. Note that when this call returns, the HAL will scan the various lists the driver
    ///	maintains (such as the device list) to get the inital set of objects the driver is
    ///	publishing. So, there is no need to notifiy the HAL about any objects created as part of the
    ///	execution of this method.
    unsafe extern "C" fn initialize(
        driver: AudioServerPlugInDriverRef,
        host: AudioServerPlugInHostRef,
    ) -> OSStatus;

    ///	This method is used to tell a driver that implements the Transport Manager semantics to
    ///	create an AudioEndpointDevice from a set of AudioEndpoints. Since this driver is not a
    ///	Transport Manager, we just check the arguments and return
    ///	kAudioHardwareUnsupportedOperationError.
    unsafe extern "C" fn create_device(
        driver: AudioServerPlugInDriverRef,
        desc: CFDictionaryRef,
        client_info: *const AudioServerPlugInClientInfo,
        device_object_id: *mut AudioObjectID,
    ) -> OSStatus;

    ///	This method is used to tell a driver that implements the Transport Manager semantics to
    ///	destroy an AudioEndpointDevice. Since this driver is not a Transport Manager, we just check
    ///	the arguments and return kAudioHardwareUnsupportedOperationError.
    unsafe extern "C" fn destroy_device(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
    ) -> OSStatus;

    ///	This method is used to inform the driver about a new client that is using the given device.
    ///	This allows the device to act differently depending on who the client is. This driver does
    ///	not need to track the clients using the device, so we just check the arguments and return
    ///	successfully.
    unsafe extern "C" fn add_device_client(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        client_info: *const AudioServerPlugInClientInfo,
    ) -> OSStatus;

    ///	This method is used to inform the driver about a client that is no longer using the given
    ///	device. This driver does not track clients, so we just check the arguments and return
    ///	successfully.
    unsafe extern "C" fn remove_device_client(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        client_info: *const AudioServerPlugInClientInfo,
    ) -> OSStatus;

    ///	This method is called to tell the device that it can perform the configuation change that it
    ///	had requested via a call to the host method, RequestDeviceConfigurationChange(). The
    ///	arguments, inChangeAction and inChangeInfo are the same as what was passed to
    ///	RequestDeviceConfigurationChange().
    ///
    ///	The HAL guarantees that IO will be stopped while this method is in progress. The HAL will
    ///	also handle figuring out exactly what changed for the non-control related properties. This
    ///	means that the only notifications that would need to be sent here would be for either
    ///	custom properties the HAL doesn't know about or for controls.
    unsafe extern "C" fn perform_device_configuration_change(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        action: u64,
        change_info: *mut c_void,
    ) -> OSStatus;

    ///	This method is called to tell the driver that a request for a config change has been denied.
    ///	This provides the driver an opportunity to clean up any state associated with the request.
    ///	For this driver, an aborted config change requires no action. So we just check the arguments
    ///	and return
    unsafe extern "C" fn abort_device_configuration_change(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        action: u64,
        change_info: *mut c_void,
    ) -> OSStatus;
    ///	This method returns whether or not the given object has the given property.
    unsafe extern "C" fn has_property(
        driver: AudioServerPlugInDriverRef,
        object_id: AudioObjectID,
        client_pid: pid_t,
        property_address: *const AudioObjectPropertyAddress,
    ) -> u8;

    ///	This method returns whether or not the given property on the object can have its value
    ///	changed.
    unsafe extern "C" fn is_property_settable(
        driver: AudioServerPlugInDriverRef,
        object_id: AudioObjectID,
        client_pid: pid_t,
        property_address: *const AudioObjectPropertyAddress,
        out: *mut u8,
    ) -> OSStatus;

    unsafe extern "C" fn get_property_data_size(
        driver: AudioServerPlugInDriverRef,
        object_id: AudioObjectID,
        client_pid: pid_t,
        property_address: *const AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const c_void,
        out: *mut u32,
    ) -> OSStatus;

    unsafe extern "C" fn get_property_data(
        driver: AudioServerPlugInDriverRef,
        object_id: AudioObjectID,
        client_pid: pid_t,
        property_address: *const AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const c_void,
        data_size: u32,
        out_size: *mut u32,
        out_data: *mut c_void,
    ) -> OSStatus;

    unsafe extern "C" fn set_property_data(
        driver: AudioServerPlugInDriverRef,
        object_id: AudioObjectID,
        client_pid: pid_t,
        property_address: *const AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const c_void,
        data_size: u32,
        to_write: *const c_void,
    ) -> OSStatus;

    unsafe extern "C" fn start_io(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        client_id: u32,
    ) -> OSStatus;

    unsafe extern "C" fn stop_io(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        client_id: u32,
    ) -> OSStatus;

    unsafe extern "C" fn get_zero_time_stamp(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        client_id: u32,
        out_sample_time: *mut f64,
        out_host_time: *mut u64,
        out_seed: *mut u64,
    ) -> OSStatus;

    unsafe extern "C" fn will_do_io_operation(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        client_id: u32,
        operation_id: u32,
        out_will_do: *mut u8,          /* bool */
        out_will_do_in_place: *mut u8, /* bool */
    ) -> OSStatus;

    unsafe extern "C" fn begin_io_operation(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        client_id: u32,
        operation_id: u32,
        io_buffer_frame_size: u32,
        io_cycle_info: *const AudioServerPlugInIOCycleInfo,
    ) -> OSStatus;

    unsafe extern "C" fn do_io_operation(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        stream_id: AudioObjectID,
        client_id: u32,
        operation_id: u32,
        io_buffer_frame_size: u32,
        io_cycle_info: *const AudioServerPlugInIOCycleInfo,
        io_main_buffer: *mut c_void,
        io_secondary_buffer: *mut c_void,
    ) -> OSStatus;

    unsafe extern "C" fn end_io_operation(
        driver: AudioServerPlugInDriverRef,
        device_id: AudioObjectID,
        client_id: u32,
        operation_id: u32,
        io_buffer_frame_size: u32,
        io_cycle_info: *const AudioServerPlugInIOCycleInfo,
    ) -> OSStatus;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PluginHostInterface {
    inner: NonNull<AudioServerPlugInHostInterface>,
}

impl PluginHostInterface {
    pub fn new(inner: *const AudioServerPlugInHostInterface) -> Option<Self> {
        Some(Self {
            inner: NonNull::new(inner.cast_mut())?,
        })
    }
    //TODO add doc commends from Apple headers
    pub unsafe fn properties_changed<'a>(
        &self,
        in_object_id: AudioObjectID,
        /* the lifetime required of this reference should be investigated */
        properties: &'a [AudioObjectPropertyAddress],
    ) -> crate::os_err::OSStatus {
        let Some(f) = (unsafe { ptr::read(self.inner.as_ptr().cast_const()).PropertiesChanged })
        else {
            return Err(OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        };

        result_from_raw((f)(
            self.inner.as_ptr().cast_const(),
            in_object_id,
            properties
                .len()
                .try_into()
                .replace_err(OSStatusError::HW_BAD_PROPERTY_SIZE_ERR)?,
            properties.as_ptr(),
        ))
    }
    pub unsafe fn copy_from_storage(
        &self,
        in_key: CFStringRef,
        out_data: *mut CFPropertyListRef,
    ) -> crate::os_err::OSStatus {
        let Some(f) = (unsafe { ptr::read(self.inner.as_ptr().cast_const()).CopyFromStorage })
        else {
            return Err(OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        };
        result_from_raw((f)(
            self.inner.as_ptr().cast_const(),
            in_key.cast(),
            out_data,
        ))
    }
    pub unsafe fn write_to_storage(
        &self,
        in_key: CFStringRef,
        in_data: CFPropertyListRef,
    ) -> crate::os_err::OSStatus {
        let Some(f) = (unsafe { ptr::read(self.inner.as_ptr().cast_const()).WriteToStorage })
        else {
            return Err(OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        };
        result_from_raw((f)(
            self.inner.as_ptr().cast_const(),
            in_key.cast(),
            in_data,
        ))
    }
    pub unsafe fn delete_from_storage(&self, in_key: CFStringRef) -> crate::os_err::OSStatus {
        let Some(f) = (unsafe { ptr::read(self.inner.as_ptr().cast_const()).DeleteFromStorage })
        else {
            return Err(OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        };
        result_from_raw((f)(self.inner.as_ptr().cast_const(), in_key.cast()))
    }
    pub unsafe fn request_device_configuration_change(
        &self,
        in_device_object_id: AudioObjectID,
        in_change_action: u64,
        in_change_info: *mut c_void,
    ) -> crate::os_err::OSStatus {
        let Some(f) = (unsafe {
            ptr::read(self.inner.as_ptr().cast_const()).RequestDeviceConfigurationChange
        }) else {
            return Err(OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        };

        result_from_raw((f)(
            self.inner.as_ptr().cast_const(),
            in_device_object_id,
            in_change_action,
            in_change_info,
        ))
    }
}
