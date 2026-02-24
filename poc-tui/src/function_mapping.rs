use std::{collections::HashMap, fs};

use color_eyre::eyre;

#[derive(Debug)]
pub struct FunctionMapping {
    pub name_to_meta: HashMap<String, FunctionMetadata>,
}

#[derive(Debug)]
pub struct FunctionMetadata {
    /// The full symbol that is assigned to the function
    pub symbol: String,
    /// The address of the function relative to the base memory
    /// of the JIT-compiled wasm binary
    pub addr: u64,
    /// The size of the function
    // TODO(aeryz): we might use this info to trace the ret instructions
    // need to check if cranelift jmps to end and then rets or do rets
    // at arbitrary locations tho.
    pub size: u64,
}

impl FunctionMapping {
    pub fn generate_from_perfmap_file_with_pid(bin_name: &str, pid: u32) -> eyre::Result<Self> {
        let mut name_to_meta = HashMap::new();

        // TODO: make this configurable
        let data = fs::read_to_string(format!("/tmp/perf-{pid}.map"))?;

        for line in data.lines() {
            // Example: "7f3a1c400000 00000034 world"
            let mut it = line.split_whitespace();
            let addr = it.next();
            let size = it.next();
            let name = it.next();

            if let (Some(addr), Some(size), Some(name)) = (addr, size, name) {
                if name.starts_with(bin_name) {
                    let addr = u64::from_str_radix(addr.trim_start_matches("0x"), 16)?;
                    let size = u64::from_str_radix(size, 16)?;

                    let maybe_name = name.split(":").last().unwrap_or(name);

                    let _ = name_to_meta.insert(
                        maybe_name.into(),
                        FunctionMetadata {
                            symbol: name.into(),
                            addr,
                            size,
                        },
                    );
                }
            }
        }

        Ok(FunctionMapping { name_to_meta })
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionMetadata> {
        self.name_to_meta.get(name)
    }
}

impl<'a> IntoIterator for &'a FunctionMapping {
    type Item = <&'a HashMap<String, FunctionMetadata> as IntoIterator>::Item;

    type IntoIter = <&'a HashMap<String, FunctionMetadata> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.name_to_meta).into_iter()
    }
}
