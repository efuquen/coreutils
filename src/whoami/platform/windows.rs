/*
 * This file is part of the uutils coreutils package.
 *
 * (c) Jordi Boggiano <j.boggiano@seld.be>
 *
 * For the full copyright and license information, please view the LICENSE
 * file that was distributed with this source code.
 */

extern crate winapi;
extern crate advapi32;

#[macro_use]
extern crate uucore;

use std::ffi::OsString;
use std::io::{Error, Result, Write};
use std::mem;
use std::os::windows::ffi::OsStringExt;
use uucore::wide::FromWide;

pub unsafe fn getusername() -> Result<String> {
    let mut buffer: [winapi::WCHAR; winapi::UNLEN as usize + 1] = mem::uninitialized();
    let mut len = buffer.len() as winapi::DWORD;
    if advapi32::GetUserNameW(buffer.as_mut_ptr(), &mut len) == 0 {
        return Err(Error::last_os_error())
    }
    let username = String::from_wide(&buffer[..len as usize - 1]);
    Ok(username)
}
