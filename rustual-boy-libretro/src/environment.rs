use callbacks::Callbacks;

const EXPERIMENTAL: isize = 0x10000;

#[allow(dead_code)]
pub enum EnvironmentCommand {
    SetRotation = 1,
    GetOverscan = 2,
    GetCanDupe = 3,
    SetMessage = 6,
    Shutdown = 7,
    SetPerformanceLevel = 8,
    GetSystemDirectory = 9,
    SetPixelFormat = 10,
    SetInputDescriptors = 11,
    SetKeyboardCallback = 12,
    SetDiskControlInterface = 13,
    SetHwRender = 14,
    GetVariable = 15,
    SetVariables = 16,
    GetVariableUpdate = 17,
    SetSupportNoGame = 18,
    GetLibretroPath = 19,
    SetAudioCallback = 22,
    SetFrameTimeCallback = 21,
    GetRumbleInterface = 23,
    GetInputDeviceCapabilities = 24,
    GetSensorInterface = (25 | EXPERIMENTAL),
    GetCameraInterface = (26 | EXPERIMENTAL),
    GetLogInterface = 27,
    GetPerfInterface = 28,
    GetLocationInterface = 29,
    GetCoreAssetsDirectory = 30,
    GetSaveDirectory = 31,
    SetSystemAvInfo = 32,
    SetProcAddressCallback = 33,
    SetSubsystemInfo = 34,
    SetControllerInfo = 35,
    SetMemoryMaps = (36 | EXPERIMENTAL),
    SetGeometry = 37,
    GetUsername = 38,
    GetLanguage = 39,
    GetCurrentSoftwareFramebuffer = (40 | EXPERIMENTAL),
    GetHwRenderInterface = (41 | EXPERIMENTAL),
    SetSupportAchievements = (42 | EXPERIMENTAL),
    SetHwRenderContextNegotiationInterface = (43 | EXPERIMENTAL),
    SetSerializationQuirks = 44,
}

#[repr(C)]
pub struct FrameTimeCallback {
    pub callback: extern "C" fn(i64),
    pub reference: i64,
}

pub enum PixelFormat {
    // Only supported pixel format for now
    Xrgb8888 = 1,
}

impl Callbacks {
    pub fn set_frame_time_callback(&self, frame_time_callback: FrameTimeCallback) {
        let ptr = Box::into_raw(Box::new(frame_time_callback));
        self.environment(EnvironmentCommand::SetFrameTimeCallback as u32,
                         ptr as *mut _);

        unsafe {
            Box::from_raw(ptr);
        }
    }

    pub fn set_pixel_format(&self, format: PixelFormat) {
        let format_ptr = Box::into_raw(Box::new(format));
        self.environment(EnvironmentCommand::SetPixelFormat as u32,
                         format_ptr as *mut _);
        unsafe {
            Box::from_raw(format_ptr);
        }
    }
}
