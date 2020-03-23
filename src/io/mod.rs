use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs::File;
use crate::error;
use std::error::Error;
use std::io::{Read, Write};

pub(crate) struct IOManager {
    open_files: AtomicUsize,
    max_open_files: usize
}

pub(crate) struct FileQuota<'a>(&'a IOManager);

impl<'a> FileQuota<'a> {
    pub(crate) fn read_file(self, file_name: String) -> Result<Vec<u8>, error::Error> {
        self.read_file_impl(&file_name).or_else(
            |e| {
                Err(error::Error::io_error(e.to_string().into(),
                                           file_name))
            }
        )
    }

    pub(crate) fn write_file(self, file_name: String, data: &[u8]) -> Result<(), error::Error> {
        self.write_file_impl(&file_name, data).or_else(
            |e| {
                Err(error::Error::io_error(e.to_string().into(),
                                           file_name))
            }
        )
    }

    fn read_file_impl(self, file_name: &String) -> Result<Vec<u8>, std::io::Error> {
        let mut v = Vec::new();
        File::with_options()
            .read(true)
            .write(false)
            .open(file_name)?
            .read_to_end(&mut v)?;
        Ok(v)
    }

    fn write_file_impl(self, file_name: &String, data: &[u8]) -> Result<(), std::io::Error> {
        File::with_options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_name)?
            .write_all(data)?;
        Ok(())
    }
}

impl<'a> Drop for FileQuota<'a> {
    fn drop(&mut self) {
        let FileQuota(io_manager) = self;
        io_manager.on_file_returned()
    }
}

impl IOManager {
    pub fn new(max_open_files: usize) -> Self {
        Self { open_files: AtomicUsize::new(0), max_open_files }
    }

    pub fn require_quota(&self) -> FileQuota {
        while self.open_files.load(Ordering::SeqCst) >= self.max_open_files {}
        self.open_files.fetch_add(1, Ordering::SeqCst);
        FileQuota(self)
    }

    fn on_file_returned(&self) {
        self.open_files.fetch_sub(1, Ordering::SeqCst);
    }
}
