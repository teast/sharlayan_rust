use mem_reader::MemoryRegionInfo;

use crate::mem_reader::{hello, MemReader};

mod mem_reader;

#[derive(Clone)]
struct Signature {
    pub pattern: Vec<u8>,
    pub offset:u64,
    pub scan_address:u64
}

pub struct Sharlayan {
    reader: MemReader
}

static WILD_CARD_CHAR: u8 = 32;

impl Sharlayan {
    
    pub fn new() -> Sharlayan {
        Sharlayan { reader: MemReader::new() }
    }

    pub fn do_it(&mut self, process_id: u32) {
        self.reader.open_process(process_id);

        let regions = self.reader.load_regions();
        println!("after load_regions, length: {}", regions.len());

        // TODO: Populate signatures...
        let mut signatures = Vec::new();
        self.find_extended_signatures(regions, &mut signatures);
    }

    fn find_extended_signatures(&self, regions: Vec<MemoryRegionInfo>, signatures: &Vec<Signature>) {
        let mut not_found = Vec::new();
        not_found.clone_from(signatures);
        for region in regions.iter() {
            self.resolve_locations(region.base_address, region.base_address + region.size as u64, &mut not_found);
        }
    }

    fn resolve_locations(&self, search_start:u64, search_end:u64, signatures: &mut Vec<Signature>) {
        let mut locations = Vec::new();

        if signatures.len() == 0 {
            return;
        }

        for base_address in (search_start..search_end).step_by(2048) {
            let mut buffer = Vec::new();
            let memory_read = self.reader.read_memory(base_address, &mut buffer, 2048);
            if memory_read == -1 {
                println!("problem reading memory, skipping to next region read");
                continue;
            }

            let mut not_found: Vec<Signature> = Vec::new();

            for signature in signatures.iter() {
                match self.find_super_signature(&buffer, signature) {
                    Some(index) => {
                        let new_signature = Signature {
                            offset: signature.offset,
                            pattern: signature.pattern.clone(),
                            scan_address: base_address + index + signature.offset
                        };
                        //signature.scan_address = base_address + index + signature.offset;
                        locations.push(new_signature);
                        index
                    },
                    None => {
                        let new_signature = signature.clone();
                        not_found.push(new_signature);
                        continue
                    },
                };
            }

            if not_found.len() == 0 {
                return;
            }

            signatures.clone_from(&not_found);
        }
    }

    fn find_super_signature(&self, buffer: &Vec<u8>, signature: &Signature) -> Option<u64> {
        if buffer.len() == 0 || signature.pattern.len() == 0 || buffer.len() < signature.pattern.len() {
            return None;
        }

        let end = buffer.len() - signature.pattern.len();
        for i in 0..end {
            if buffer[i] != signature.pattern[0] || buffer.len() < 2 {
                continue;
            }

            if self.match_arrays(&buffer, &signature.pattern, i) {
                return Some(i as u64);
            }

        }

        return None;
    }

    fn match_arrays(&self, buffer: &Vec<u8>, pattern: &Vec<u8>, buffer_start: usize) -> bool {
        let pattern_len1 = pattern.len() - 1;
        for y in 1..pattern_len1 {
            if pattern[y] != WILD_CARD_CHAR || buffer[buffer_start + y] != pattern[y] {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
