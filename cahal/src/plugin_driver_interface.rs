use core_foundation::{
    base::{kCFAllocatorDefault, CFAllocatorRef, CFEqual, CFRelease, CFRetain},
    uuid::{CFUUIDCreateFromUUIDBytes, CFUUIDGetConstantUUIDWithBytes, CFUUIDRef},
};
use coreaudio_sys::{
    kAudioHardwareIllegalOperationError, AudioServerPlugInDriverInterface, REFIID,
};
use log::{error, info, warn};
use std::{
    cell::OnceCell,
    mem::transmute,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
    thread::LocalKey,
};

use crate::{
    os_err::result_to_err_code,
    raw_plugin_driver_interface::{PluginHostInterface, RawAudioServerPlugInDriverInterface},
};

/// ## Audio Server Plugin Interface
///
/// This is the interface that contains all of the functions CoreAudio needs to interact with your driver.
///
/// #### Real Time safety
/// This interface contains *real time* functions. These functions (marked in documentation comments) MUST run in constant time.
/// ANY lag spikes will cause undesirable audio glitches
/// * Code in the `*_io` and `*_io_operation` functions on this trait must execute in constant time
/// * Functions will be marked as requiring real time safe execution on a best effort basis as I investigate CoreAudio further
///
/// #### Thread Safety
/// Functions in this interface implementation can be called from any thread, at any time, at the disgression of the OS
/// and its client applications. As such, functions are passed a *shared* reference to the implementation's global state and it is required that that state implement `Sync`.
///
pub trait AudioServerPluginDriverInterface {
    /// The type (likely either an enum or `()`) used to communicate changes in device state through the CoreAudio HAL machinery
    type DeviceConfigurationChangeInfo: Send;
    const NAME: &'static str;
    /// This is the constructor of your driver. You will probably want to allocate resources here, as this is the last time you will have exclusive access to global state
    fn create(cf_allocator: CFAllocatorRef) -> Self;
    /// This function is called when the HAL tries to bring your driver up, this is where you'll want to do any complex computation, and query the host for information via the host interface
    /// You're also expected to store the host interface for later use.
    fn init(&self, host: PluginHostInterface<Self>) -> crate::os_err::OSStatus;
}

#[repr(C)]
pub struct PluginDriverImplementation<T> {
    implementation: *const AudioServerPlugInDriverInterface,
    state: T,
    refcount: AtomicU32,
}
macro_rules! validate_impl_ref {
    ($ptr:expr) => {{
        let Some(f) = $ptr.cast::<PluginDriverImplementation<Self>>().as_ref() else {
            return ::coreaudio_sys::kAudioHardwareIllegalOperationError as i32;
        };
        f
    }};
}
// Until this is fully implemented
#[allow(unused_variables)]
impl<Implementation> RawAudioServerPlugInDriverInterface for Implementation
where
    Implementation: Sync + AudioServerPluginDriverInterface + 'static,
{
    unsafe extern "C" fn create(
        alloc: coreaudio_sys::CFAllocatorRef,
        requested_uuid: crate::base::CFUUIDRef,
    ) -> *mut std::ffi::c_void {
        let mut logger = oslog::OsLogger::new(&format!("com.rustaudio.{}", Self::NAME));

        #[cfg(not(debug_assertions))]
        {
            logger = logger.level_filter(log::LevelFilter::Error);
        }
        #[cfg(debug_assertions)]
        {
            logger = logger.level_filter(log::LevelFilter::Trace);
        }

        let Ok(()) = logger.init() else {
            panic!("failed to initialize logger from Rust CoreAudio Driver");
        };

        info!("Driver Plugin Driver Constructor: {}", Self::NAME);
        if unsafe {
            CFEqual(
                requested_uuid.cast(),
                AUDIO_SERVER_DRIVER_PLUGIN_TYPE.get().cast(),
            ) == 1
        } {
            // Init and allocate driver
            let driver_state = Implementation::create(alloc.cast());

            // explicitly borrow IMPLEMENTATION for 'static (to make it clear that it gets promoted to a static)
            let impl_borrow: &'static AudioServerPlugInDriverInterface = &Self::IMPLEMENTATION;

            // Allocate implementation container
            Box::<_>::into_raw(Box::new(PluginDriverImplementation {
                implementation: impl_borrow as *const AudioServerPlugInDriverInterface,
                refcount: AtomicU32::new(1),
                state: driver_state,
            }))
            .cast()
        } else {
            ptr::null_mut()
        }
    }

    unsafe extern "C" fn query_interface(
        driver: *mut std::ffi::c_void,
        in_uuid: coreaudio_sys::REFIID,
        out_interface: *mut coreaudio_sys::LPVOID,
    ) -> coreaudio_sys::HRESULT {
        if out_interface.is_null() {
            error!("no space for output of query_interface");
            return kAudioHardwareIllegalOperationError as i32;
        }
        let requested_uuid = unsafe {
            CFUUIDCreateFromUUIDBytes(
                ptr::null_mut(),
                transmute::<REFIID, core_foundation::uuid::CFUUIDBytes>(in_uuid),
            )
        };
        if requested_uuid.is_null() {
            error!("failed to create new uuid from device in {}", file!());
            return kAudioHardwareIllegalOperationError as i32;
        }
        //HRESULT ok
        let mut ret = 0;
        if unsafe {
            CFEqual(
                requested_uuid.cast(),
                AUDIO_SERVER_DRIVER_PLUGIN_INTERFACE.get().cast(),
            ) == 1
                || CFEqual(requested_uuid.cast(), I_UNKNOWN_INTERFACE.get().cast()) == 1
        } {
            info!("query interface matched");
            unsafe { ptr::write(out_interface, driver) }
        } else {
            // E_NOINTERFACE, CFPlugInCOM.h
            ret = 0x80000004u32 as i32;
            warn!("Requested interface did not match in QueryInterface!");
        }
        unsafe { CFRelease(requested_uuid.cast()) }
        ret
    }

    unsafe extern "C" fn retain(driver: *mut std::ffi::c_void) -> coreaudio_sys::ULONG {
        let Some(r) = driver.cast::<PluginDriverImplementation<Self>>().as_ref() else {
            //0 refcount for null implementation
            error!("attempted to retain null implementation");
            return 0;
        };
        // Add the reference we added
        let prev_count = r.refcount.fetch_add(1, Ordering::SeqCst);
        info!("retain called, new refcount: {}", prev_count + 1);
        //Pointer is non-null, refcount is 0
        if prev_count == 0 {
            0
        } else {
            // Do the increment we just did on the previous value
            prev_count + 1
        }
    }

    unsafe extern "C" fn release(driver: *mut std::ffi::c_void) -> coreaudio_sys::ULONG {
        // We are not actually supposed to deallocate anything when this reaches 0 for whatever reason
        let Some(r) = driver.cast::<PluginDriverImplementation<Self>>().as_ref() else {
            warn!("attempted to release null implementation");
            //0 refcount for null implementation
            return 0;
        };

        let ret = r
            .refcount
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |val| {
                Some(val.saturating_sub(1))
            })
            .unwrap()
            .saturating_sub(1);
        info!("release called, new refcount: {}", ret);
        ret
    }
    unsafe extern "C" fn initialize(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        host: coreaudio_sys::AudioServerPlugInHostRef,
    ) -> coreaudio_sys::OSStatus {
        info!("Initialize called: {}", Self::NAME);
        let Some(hostref) = PluginHostInterface::new(host) else {
            return kAudioHardwareIllegalOperationError as i32;
        };
        let implementation = validate_impl_ref!(driver);
        result_to_err_code(implementation.state.init(hostref))
    }

    unsafe extern "C" fn create_device(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        desc: coreaudio_sys::CFDictionaryRef,
        client_info: *const coreaudio_sys::AudioServerPlugInClientInfo,
        device_object_id: *mut coreaudio_sys::AudioObjectID,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn destroy_device(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn add_device_client(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        client_info: *const coreaudio_sys::AudioServerPlugInClientInfo,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn remove_device_client(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        client_info: *const coreaudio_sys::AudioServerPlugInClientInfo,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn perform_device_configuration_change(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        action: u64,
        change_info: *mut std::ffi::c_void,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn abort_device_configuration_change(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        action: u64,
        change_info: *mut std::ffi::c_void,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn has_property(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        object_id: coreaudio_sys::AudioObjectID,
        client_pid: coreaudio_sys::pid_t,
        property_address: *const coreaudio_sys::AudioObjectPropertyAddress,
    ) -> u8 {
        todo!()
    }

    unsafe extern "C" fn is_property_settable(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        object_id: coreaudio_sys::AudioObjectID,
        client_pid: coreaudio_sys::pid_t,
        property_address: *const coreaudio_sys::AudioObjectPropertyAddress,
        out: *mut u8,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn get_property_data_size(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        object_id: coreaudio_sys::AudioObjectID,
        client_pid: coreaudio_sys::pid_t,
        property_address: *const coreaudio_sys::AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const std::ffi::c_void,
        out: *mut u32,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn get_property_data(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        object_id: coreaudio_sys::AudioObjectID,
        client_pid: coreaudio_sys::pid_t,
        property_address: *const coreaudio_sys::AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const std::ffi::c_void,
        data_size: u32,
        out_size: *mut u32,
        out_data: *mut std::ffi::c_void,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn set_property_data(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        object_id: coreaudio_sys::AudioObjectID,
        client_pid: coreaudio_sys::pid_t,
        property_address: *const coreaudio_sys::AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const std::ffi::c_void,
        data_size: u32,
        to_write: *const std::ffi::c_void,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn start_io(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        client_id: u32,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn stop_io(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        client_id: u32,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn get_zero_time_stamp(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        client_id: u32,
        out_sample_time: *mut f64,
        out_host_time: *mut u64,
        out_seed: *mut u64,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn will_do_io_operation(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        client_id: u32,
        operation_id: u32,
        out_will_do: *mut u8,          /* bool */
        out_will_do_in_place: *mut u8, /* bool */
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn begin_io_operation(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        client_id: u32,
        operation_id: u32,
        io_buffer_frame_size: u32,
        io_cycle_info: *const coreaudio_sys::AudioServerPlugInIOCycleInfo,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn do_io_operation(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        stream_id: coreaudio_sys::AudioObjectID,
        client_id: u32,
        operation_id: u32,
        io_buffer_frame_size: u32,
        io_cycle_info: *const coreaudio_sys::AudioServerPlugInIOCycleInfo,
        io_main_buffer: *mut std::ffi::c_void,
        io_secondary_buffer: *mut std::ffi::c_void,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }

    unsafe extern "C" fn end_io_operation(
        driver: coreaudio_sys::AudioServerPlugInDriverRef,
        device_id: coreaudio_sys::AudioObjectID,
        client_id: u32,
        operation_id: u32,
        io_buffer_frame_size: u32,
        io_cycle_info: *const coreaudio_sys::AudioServerPlugInIOCycleInfo,
    ) -> coreaudio_sys::OSStatus {
        todo!()
    }
}
const fn uuid_u128(uuid: [u8; 16]) -> u128 {
    u128::from_le_bytes(uuid)
}
thread_local! {
    static I_UNKNOWN_INTERFACE: StaticUUID<
            {
                uuid_u128([
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x46,
                ])
            },
        > = const { StaticUUID::new() };
    static AUDIO_SERVER_DRIVER_PLUGIN_INTERFACE: StaticUUID<
            {
                uuid_u128([
                    0xEE, 0xA5, 0x77, 0x3D, 0xCC, 0x43, 0x49, 0xF1, 0x8E, 0x00, 0x8F, 0x96, 0xE7, 0xD2, 0x3B,
                    0x17,
                ])
            }
        > = const { StaticUUID::new() };
    static AUDIO_SERVER_DRIVER_PLUGIN_TYPE: StaticUUID<
            {
                uuid_u128([
                    0x44, 0x3A, 0xBA, 0xB8, 0xE7, 0xB3, 0x49, 0x1A, 0xB9, 0x85, 0xBE, 0xB9, 0x18, 0x70, 0x30,
                    0xDB,
                ])
            }
        > = const { StaticUUID::new() };

}

struct StaticUUID<const UUID: u128> {
    cache: OnceCell<CFUUIDRef>,
}
impl<const UUID: u128> StaticUUID<UUID> {
    const fn new() -> Self {
        Self {
            cache: OnceCell::new(),
        }
    }
    fn get(&self) -> CFUUIDRef {
        *self.cache.get_or_init(|| {
            let uuid = UUID.to_le_bytes();
            let r = unsafe {
                CFUUIDGetConstantUUIDWithBytes(
                    kCFAllocatorDefault,
                    uuid[0],
                    uuid[1],
                    uuid[2],
                    uuid[3],
                    uuid[4],
                    uuid[5],
                    uuid[6],
                    uuid[7],
                    uuid[8],
                    uuid[9],
                    uuid[10],
                    uuid[11],
                    uuid[12],
                    uuid[13],
                    uuid[14],
                    uuid[15],
                )
            };
            unsafe { CFRetain(r.cast()) }.cast()
        })
    }
}
trait LocalKeyExt {
    fn get(&'static self) -> CFUUIDRef;
}
impl<const UUID: u128> LocalKeyExt for LocalKey<StaticUUID<UUID>> {
    fn get(&'static self) -> CFUUIDRef {
        self.with(|v| v.get())
    }
}
