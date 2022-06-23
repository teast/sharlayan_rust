#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
#[cfg_attr(target_os = "macos", path = "macos.rs")]
mod real;

use crate::mem_reader::real::*;

pub struct MemoryRegionInfo {
    pub base_address: u64,
    pub size: usize
}

pub struct MemReader {
    handler: OsHandler
}

impl MemReader {
    pub fn new() -> MemReader {
        MemReader { handler: real::OsHandler::new() }
    }

    pub fn open_process(&mut self, process_id: u32) {
        self.handler.open_process(process_id);
    }

    pub fn close_process(&mut self) {
        self.handler.close_process();
    }

    pub fn read_memory(&self, base_address:u64, buffer:&mut Vec<u8>, wanted_size: usize) -> i64 {
        self.handler.read_memory(base_address, buffer, wanted_size)
    }

    pub fn load_regions(&self) -> Vec<MemoryRegionInfo> {
        self.handler.load_regions()
    }
}
pub fn hello() {
    //real::hello();
}