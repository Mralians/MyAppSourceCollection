use humantime;
use pam_sys::{raw::pam_get_item, PamFlag, PamHandle, PamItemType, PamReturnCode};
use std::ffi::{c_void, CStr};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;

#[derive(Debug)]
struct LoginInfo {
    username: String,
    service: String,
    timestamp: humantime::Rfc3339Timestamp,
}

impl Display for LoginInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}  {}   {}\n",
            self.username, self.service, self.timestamp
        )
    }
}

fn get_pam_item(pamh: *mut PamHandle, item_type: PamItemType) -> Result<String, PamReturnCode> {
    let mut item_ptr: *const c_void = std::ptr::null();
    unsafe {
        if pam_get_item(pamh, item_type as i32, &mut item_ptr as *mut *const c_void)
            != PamReturnCode::SUCCESS as i32
        {
            return Err(match item_type {
                PamItemType::USER => PamReturnCode::USER_UNKNOWN,
                PamItemType::SERVICE => PamReturnCode::SERVICE_ERR,
                _ => PamReturnCode::SERVICE_ERR,
            });
        }
        Ok(CStr::from_ptr(item_ptr as *const i8)
            .to_string_lossy()
            .into_owned())
    }
}

fn log_login_info(login_info: &LoginInfo) -> Result<(), std::io::Error> {
    let mut log = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open("/tmp/log.txt")?;
    log.write_all(login_info.to_string().as_bytes())
}

#[no_mangle]
pub extern "C" fn pam_sm_authenticate(
    pamh: *mut PamHandle,
    _flags: PamFlag,
    _argc: i32,
    _argv: *const *const i8,
) -> PamReturnCode {
    let username = match get_pam_item(pamh, PamItemType::USER) {
        Ok(user) => user,
        Err(err) => return err,
    };

    let service = match get_pam_item(pamh, PamItemType::SERVICE) {
        Ok(svc) => svc,
        Err(err) => return err,
    };

    let timestamp = humantime::format_rfc3339_seconds(SystemTime::now());
    let login_info = LoginInfo {
        username,
        service,
        timestamp,
    };

    if log_login_info(&login_info).is_err() {
        return PamReturnCode::SERVICE_ERR;
    }

    PamReturnCode::SUCCESS
}

#[no_mangle]
pub extern "C" fn pam_sm_setcred(
    _pamh: *mut PamHandle,
    _flags: PamFlag,
    _argc: i32,
    _argv: *const *const i8,
) -> PamReturnCode {
    PamReturnCode::SUCCESS
}
