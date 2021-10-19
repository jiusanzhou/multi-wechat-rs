// query system information from ntdll

use std::ptr::null_mut;

use std::mem;
use std::io;
use std::slice;
use std::io::{Error, ErrorKind};

use crate::winapi::*;
use crate::utils::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle {
    pub pid: u32,
    pub handle: HANDLE,
    pub type_index: u32,
    pub type_name: String,
    pub name: String,
}

impl Handle {

    pub fn new(handle: HANDLE, pid: u32, type_index: u32, type_name: String, name: String) -> Self {
        Self{handle, pid, type_index, type_name, name}
    }

}

impl Handle {
    pub fn close_handle(&self) -> Result<(), io::Error> {

        // open process again
        let process = unsafe{OpenProcess(PROCESS_ALL_ACCESS, FALSE, self.pid as _)};
        if process.is_null() {
            return Err(Error::new(ErrorKind::NotFound, "pid"));
        }

        // duplicate handle to close handle
        let mut nhe: HANDLE = null_mut();
        let r = unsafe{
            DuplicateHandle(
                process, self.handle as _, GetCurrentProcess(),
                &mut nhe, 0, FALSE, DUPLICATE_CLOSE_SOURCE)};

        if r == FALSE {
            println!("duplicate handle to close failed");
            return Err(get_last_error());
        }

        Ok(())
    }
}

// TODO: add filter function
pub fn get_system_handles(pid: u32) -> Result<Vec<Handle>, io::Error>{

    // Windows 10 notebook requires at least 512KiB of memory to make it in one go
    let mut buffer_size: usize = 512 * 1024;

    let mut return_len = 0;
    let mut buf: Vec<u8>;

    loop {
        buf = Vec::with_capacity(buffer_size);

        let result = unsafe {
            NtQuerySystemInformation(
                SystemHandleInformation,
                buf.as_mut_ptr() as PVOID,
                buffer_size as u32,
                &mut return_len,
            )
        };

        if NT_SUCCESS(result) {
            break;
        }

        // if is 0xc0000004: STATUS_INFO_LENGTH_MISMATCH
        // growing the buffer
        if result != STATUS_INFO_LENGTH_MISMATCH {
            return Err(Error::new(ErrorKind::Other, format!("[{:#x}] oh no!", result)));
        }

        buffer_size *= 2;
    }

    // parse the data to handle inforamtions
    let hiptr = buf.as_ptr() as *const SYSTEM_HANDLE_INFORMATION;
    let hi = unsafe{ &*hiptr };

    // println!("get all handles: {}", hi.NumberOfHandles);

    let mut handles = Vec::new();

    let raw_handles = unsafe{ slice::from_raw_parts(hi.Handles.as_ptr(), hi.NumberOfHandles as usize) };

    for he in raw_handles {
        // TODO: filter which pid is not target one
        if pid != he.UniqueProcessId as u32 {
            continue;
        }
        
        let process = unsafe{OpenProcess(PROCESS_ALL_ACCESS, FALSE, he.UniqueProcessId as _)};
        if process.is_null() {
            continue;
        }

        // duplicate handle for query information
        let mut nhe: HANDLE = null_mut();
        let r = unsafe{
            DuplicateHandle(
                process,
                he.HandleValue as _,
                GetCurrentProcess(),
                &mut nhe,
                0, FALSE, DUPLICATE_SAME_ACCESS)};
        
        // TODO: there any defer?
        unsafe{CloseHandle(process)};

        if r == 0 {
            // println!("duplicate handle failed ?");
            continue;
        }

        // copy handle and query name and type
        let mut buf = [0u8; 1024];

        // name
        let status = unsafe{NtQueryObject(nhe, ObjectNameInformation, buf.as_mut_ptr() as _, buf.len() as _, null_mut())};
        if status != STATUS_SUCCESS {
            unsafe{CloseHandle(nhe)};
            continue;
        }
        let name = unsafe{&(&*(buf.as_ptr() as *const OBJECT_NAME_INFORMATION)).Name};
        let name = utf16_to_string(name);

        // println!("dupliate handle success: {}", name);

        buf = unsafe { ::std::mem::zeroed() };

        // type
        let status = unsafe{NtQueryObject(nhe, ObjectTypeInformation, buf.as_mut_ptr() as _, buf.len() as _, null_mut())};
        if status != STATUS_SUCCESS {
            unsafe{CloseHandle(nhe)};
            continue;
        }
        let type_name = unsafe{&(&*(buf.as_ptr() as *const OBJECT_TYPE_INFORMATION)).TypeName};
        let type_name = utf16_to_string(type_name);

        handles.push(Handle::new(he.HandleValue as HANDLE, he.UniqueProcessId as u32, he.ObjectTypeIndex as u32, type_name, name));

        unsafe{CloseHandle(nhe)};
    }
    
    Ok(handles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::*;

    #[test]
    fn check_status() {
        // check status
        assert_eq!(STATUS_INFO_LENGTH_MISMATCH as u32, 0xC0000004 as u32);

        // test box grow
        let mut buf: Box<[u8]>;
        for n in 1..=10 {
            buf = vec![0; n].into_boxed_slice();
            println!("===> n: {}", buf.len());

        }

        // test array
        let arr = [1u8; 10];
        let u8_size = mem::size_of::<u8>();
        let arr_addr = arr.as_ptr();
        for i in 0..10 {
            let iptr = unsafe{ arr_addr.offset(i as isize * u8_size as isize) as *const u8 };
            let item = unsafe { *iptr };
            println!("===> item: {}", item);
        }
    }

    #[test]
    fn get_handles() {
        match Process::find_first_by_name("WeChat.exe") {
            Some(p) => {
                match get_system_handles(p.pid()) {
                   Err(e) => eprintln!("get error: {}", e),
                   Ok(v) => {
                        println!("system information");
                        let x = v.iter()
                            .filter(|x| x.type_name == "Mutant" && x.name.contains("_WeChat_App_Instance_Identity_Mutex_Name"))
                            // .cloned()
                            .collect::<Vec<_>>();
                        for i in x {
                            println!("close Mutant handle for pid: {} {}", i.pid, i.name);
                            match i.close_handle() {
                                Ok(_) => {
                                    println!("Success!");
                                },
                                Err(err) => println!("Failed: {}", err)
                            }
                        }
                   }
                }
            },
            None => {}
        }
    }
}