//! LabVIEW runtime library integration

use std::ffi::c_void;
use std::sync::LazyLock;

use dlopen2::wrapper::{Container, WrapperApi};

use crate::labview_interop::errors::{InternalError, LVInteropError, Result};
use crate::labview_interop::memory::MagicCookie;
use crate::labview_interop::types::LVStatusCode;

const LVRT_PATH: &str = "lvrt";

fn load_container<T: WrapperApi>() -> Result<Container<T>> {
    let self_result = unsafe {
        Container::load_self()
            .map_err(|e| LVInteropError::InternalError(InternalError::NoLabviewApi(e.to_string())))
    };
    match self_result {
        Ok(container) => Ok(container),
        Err(_) => {
            unsafe {
                Container::load(LVRT_PATH).map_err(|e| {
                    LVInteropError::InternalError(InternalError::NoLabviewApi(e.to_string()))
                })
            }
        }
    }
}

static SYNC_API: LazyLock<Result<Container<SyncApi>>> = LazyLock::new(load_container);

pub fn sync_api() -> Result<&'static Container<SyncApi>> {
    SYNC_API.as_ref().map_err(|e| e.clone())
}

#[derive(WrapperApi)]
pub struct SyncApi {
    #[dlopen2_name = "PostLVUserEvent"]
    post_lv_user_event:
        unsafe extern "C" fn(reference: MagicCookie, data: *mut c_void) -> LVStatusCode,

    #[dlopen2_name = "Occur"]
    occur: unsafe extern "C" fn(occurrence: MagicCookie) -> LVStatusCode,
}