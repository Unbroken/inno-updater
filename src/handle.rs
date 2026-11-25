/*-----------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *----------------------------------------------------------------------------------------*/

use std::ffi::c_void;
use std::path::Path;
use std::{error, io, ptr};
use crate::strings::to_u16s;
use crate::util;
use windows_sys::Win32::Foundation::HANDLE;

pub struct FileHandle(HANDLE);

impl FileHandle {
	pub fn new(path: &Path) -> Result<FileHandle, Box<dyn error::Error>> {
		use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
		use windows_sys::Win32::Storage::FileSystem::{
			CreateFileW, DELETE, FILE_ATTRIBUTE_NORMAL, OPEN_EXISTING,
		};

		unsafe {
			let path_wide = to_u16s(path.as_os_str());
			let handle = CreateFileW(
				path_wide.as_ptr(),
				DELETE,
				0,
				ptr::null_mut(),
				OPEN_EXISTING,
				FILE_ATTRIBUTE_NORMAL,
				std::mem::zeroed(),
			);

			if handle == INVALID_HANDLE_VALUE {
				return Err(io::Error::last_os_error().into());
			}

			Ok(FileHandle(handle))
		}
	}

	pub fn mark_for_deletion(&self) -> Result<(), Box<dyn error::Error>> {
		self.set_delete_disposition(true)
	}

	/// Keeps a file that was marked for deletion when its handle closes.
	pub fn unmark_for_deletion(&self) -> Result<(), Box<dyn error::Error>> {
		self.set_delete_disposition(false)
	}

	fn set_delete_disposition(&self, delete: bool) -> Result<(), Box<dyn error::Error>> {
		use std::mem;
		use windows_sys::Win32::Foundation::BOOLEAN;
		use windows_sys::Win32::Storage::FileSystem::{
			FileDispositionInfo, SetFileInformationByHandle, FILE_DISPOSITION_INFO,
		};

		unsafe {
			let mut info = FILE_DISPOSITION_INFO {
				DeleteFile: delete as BOOLEAN,
			};
			let result = SetFileInformationByHandle(
				self.0,
				FileDispositionInfo,
				&mut info as *mut _ as *mut c_void,
				mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
			);

			// SetFileInformationByHandle returns zero on failure
			if result == 0 {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					format!(
						"Failed to set whether to delete a file: {}",
						util::get_last_error_message()?
					),
				)
				.into());
			}
		}

		Ok(())
	}

	pub fn close(&self) -> Result<(), Box<dyn error::Error>> {
		use windows_sys::Win32::Foundation::CloseHandle;

		unsafe {
			// CloseHandle returns zero on failure
			if CloseHandle(self.0) == 0 {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					format!(
						"Failed to close file handle: {}",
						util::get_last_error_message()?
					),
				)
				.into());
			}
		}

		Ok(())
	}
}
