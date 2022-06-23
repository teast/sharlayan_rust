use std::{ffi::c_void, mem};

use windows::{Win32::{UI::WindowsAndMessaging::*, System::{Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS}, Diagnostics::Debug::ReadProcessMemory, Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_READWRITE, PAGE_WRITECOPY, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD}}, Foundation::{HANDLE, CloseHandle}}};

use super::MemoryRegionInfo;

pub struct OsHandler {
    handler: HANDLE
}

impl OsHandler {
    pub fn hello() {
        unsafe {
            MessageBoxA(None, "Hello world from windows", "Hello", MB_OK);
        };
    }

    pub fn new() -> OsHandler {
        OsHandler { handler: HANDLE(0) }
    }
    
    pub fn open_process(&mut self, process_id:u32) {
        match self.impl_open_process(process_id) {
            Ok(handle) => self.handler = handle,
            Err(e) => println!("error open process: {:?}", e),
        }
    }

    pub fn close_process(&mut self) {
        if self.handler.is_invalid() {
            return;
        }

        if self.impl_close_handle(self.handler) {
            self.handler = HANDLE(0);
        }
    }

    pub fn read_memory(&self, base_address:u64, buffer:&mut Vec<u8>, wanted_size: usize) -> i64 {
        if self.handler.is_invalid() {
            return -1;
        }

        self.impl_read_memory(self.handler, base_address, buffer, wanted_size)
    }

    pub fn load_regions(&self) -> Vec<MemoryRegionInfo> {
        let mut regions = Vec::new();

        if self.handler.is_invalid() {
            return regions;
        }

        // TODO: Check how .net loads process.Process.MainModule.BaseAddress
        //regions.push(MemoryRegionInfo {
        //    base_address: process.Process.MainModule.BaseAddress,
        //    size: process.Process.MainModule.ModuleMemorySize,
        //});
        
        let mut running = true;

        let mut address: u64 = 0;
        let mut counter = 0;

        while running == true {
            let mut info = MEMORY_BASIC_INFORMATION::default();
            let result = self.impl_virtual_query(self.handler, address, &mut info);
            if result == 0 {
                println!("virtual_query result equals 0");
                running = false;
                break;
            }

            // TODO: check if system module
            let size = match u64::try_from(info.RegionSize) {
                Ok(val) => val,
                Err(e) => {
                    println!("got error from parsing regionSize... {:?}", e);
                    return regions;
                }
            };

            if (info.State & MEM_COMMIT).0 != 0 &&
               (info.Protect & (PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_WRITECOPY | PAGE_GUARD)).0 != 0 &&
                (info.Protect & PAGE_GUARD).0 == 0 {

                println!("got something back from {}, {:?}", address, info);

                regions.push(MemoryRegionInfo {
                    base_address: info.BaseAddress as u64,
                    size: size as usize,
                });
            }

            if (u64::MAX - (info.BaseAddress as u64)) < size {
                println!("aborting due to overflow");
                running = false;
                break;
            }

            address = (info.BaseAddress as u64) + size;
            counter = counter + 1;

            if counter > 5000 {
                println!("Aborting due to soft break at 10");
                running = false;
                break;
            }
        }

        return regions;
    }










    fn impl_open_process(&self, process_id:u32) -> Result<HANDLE, windows::core::Error> {
        unsafe {
            // 2035711u32 == PROCESS_VM_ALL
           return OpenProcess(PROCESS_ACCESS_RIGHTS(2035711u32), None, process_id);
        }
    }
    
    fn impl_close_handle(&self, handle:HANDLE) -> bool {
        unsafe {
            return CloseHandle(handle).as_bool();
        }
    }
    
    fn impl_read_memory(&self, process: HANDLE, base_address:u64, buffer:&mut Vec<u8>, wanted_size:usize) -> i64 {
        unsafe {
            let mut number_of_bytes: usize = 0;
            let mut cbuffer = [0u8; 2048];
            let cpointer = &mut cbuffer as *mut [u8; 2048];
            let actual_wanted_size = match wanted_size {
                n if (1..2048).contains(&n) => wanted_size,
                _ => 2048
            };

            let result = ReadProcessMemory(process, 
                base_address as *const c_void,
                cpointer as *mut c_void, actual_wanted_size, &mut number_of_bytes as *mut usize).as_bool();
            if result == false {
                return -1;
            }

            buffer.clear();
            buffer.set_len(number_of_bytes);
            buffer.copy_from_slice(&cbuffer[0..number_of_bytes]);

    
            return number_of_bytes as i64;
        }
    }
    
    fn impl_virtual_query(&self, process: HANDLE, base_address: u64, info: *mut MEMORY_BASIC_INFORMATION) -> usize {
        unsafe {
            VirtualQueryEx(process, base_address as *const c_void, info, mem::size_of::<MEMORY_BASIC_INFORMATION>())
        }
    }
}
