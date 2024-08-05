use crate::wasi;
use std::ffi::CString;

pub struct Args {
    pub data_size: usize,
    pub data_values: Vec<CString>,
}

impl Args {
    // create new state containing environment
    pub fn new() -> Args {
        Args {
            data_size: 0,
            data_values: Vec::new(),
        }
    }

    // Return the number of environment entries and the total buffer size containing all the environment pairs, compatible with the WASI function signature.
    pub fn arg_sizes_get(&self) -> (usize, usize) {
        (self.data_values.len(), self.data_size)
    }

    // Fill up the memory with the arguments. The function is compatible with the corresponding WASI function signature.
    // entries   -   reference to the table of pointers to the buffer parts containing C-style strings in the format: name=value.
    //               It must have enough memory to fit in all the pointers.
    //
    // buffer    -   The buffer containing all the pairs. The buffer must have enough memory to fit in all the (name,value) pairs.
    pub unsafe fn arg_get(&self, entries: *mut *mut u8, buffer: *mut u8) -> wasi::Errno {
        let entries = std::slice::from_raw_parts_mut(entries, self.data_values.len());
        let buffer = std::slice::from_raw_parts_mut(buffer, self.data_size);

        let mut cursor = 0;

        for (index, elem) in self.data_values.iter().enumerate() {
            let bytes = elem.as_bytes_with_nul();
            let len = bytes.len();

            buffer[cursor..(cursor + len)].copy_from_slice(bytes);

            let pointer = buffer[cursor..(cursor + len)].as_mut_ptr();

            entries[index] = pointer;

            cursor += len;
        }

        wasi::ERRNO_SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use crate::wasi;

    use super::Args;
    use std::{ffi::CStr, ptr};
}
