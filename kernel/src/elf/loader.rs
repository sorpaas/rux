use core::fmt;
use core::mem::{size_of};
use core::slice;
use core::str;

use super::{FileHeader, ProgramHeader, SectionHeader, Symbol, StrOffset};

/// Abstract representation of a loadable ELF binary.
pub struct ElfBinary<'s> {
    name: &'s str,
    region: &'s [u8],
    header: &'s FileHeader,
}

impl<'s> fmt::Debug for ElfBinary<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.name, self.header)
    }
}

// T must be a POD for this to be safe
unsafe fn slice_pod<T>(region: &[u8], offset: usize, count: usize) -> &[T] {
    assert!(region.len() - offset >= count * size_of::<T>());
    slice::from_raw_parts(region[offset..].as_ptr() as *const T, count)
}

#[allow(dead_code)]
impl<'s> ElfBinary<'s> {
    /// Create a new ElfBinary.
    /// Makes sure that the provided region has valid ELF magic byte sequence
    /// and is big enough to contain at least the ELF file header
    /// otherwise it will return None.
    pub fn new(name: &'s str, region: &'s [u8]) -> Option<ElfBinary<'s>> {
        use super::{ELF_MAGIC};
        
        if region.len() >= size_of::<FileHeader>() && region.starts_with(ELF_MAGIC) {
            let header: &FileHeader = unsafe { &slice_pod(region, 0, 1)[0] };
            Some(ElfBinary { name: name, region: region, header: header })
        } else {
            None
        }
    }

    pub fn file_header(&self) -> &'s FileHeader {
        self.header
    }

    /// Create a slice of the program headers.
    pub fn program_headers(&self) -> &'s [ProgramHeader] {
        let correct_header_size = self.header.phentsize as usize == size_of::<ProgramHeader>();
        let pheader_region_size = self.header.phoff as usize + self.header.phnum as usize * self.header.phentsize as usize;
        let big_enough_region = self.region.len() >= pheader_region_size;

        if self.header.phoff == 0 || !correct_header_size || !big_enough_region {
            return &[];
        }

        unsafe {
            slice_pod(self.region, self.header.phoff as usize, self.header.phnum as usize)
        }
    }

    // Get the string at offset str_offset in the string table strtab
    fn strtab_str(&self, strtab: &'s SectionHeader, str_offset: StrOffset) -> &'s str {
        use super::{SHT_STRTAB};
        
        assert!(strtab.shtype == SHT_STRTAB);
        let data = self.section_data(strtab);
        let offset = str_offset.0 as usize;
        let mut end = offset;
        while data[end] != 0 {
            end += 1;
        }
        str::from_utf8(&data[offset..end]).unwrap()
    }

    // Get the name of the section
    pub fn symbol_name(&self, symbol: &'s Symbol) -> &'s str {
        use super::{SHT_STRTAB};
        
        let strtab = self.section_headers().iter().find(|s| s.shtype == SHT_STRTAB && self.section_name(s) == ".strtab").unwrap();
        self.strtab_str(strtab, symbol.name)
    }

    // Get the data of the section
    pub fn section_data(&self, section: &'s SectionHeader) -> &'s [u8] {
        &self.region[(section.offset as usize)..(section.offset as usize + section.size as usize)]
    }

    // Get the name of the section
    pub fn section_name(&self, section: &'s SectionHeader) -> &'s str {
        self.strtab_str(&self.section_headers()[self.header.shstrndx as usize], section.name)
    }

    // Get the symbols of the section
    fn section_symbols(&self, section: &'s SectionHeader) -> &'s [Symbol] {
        use super::{SHT_SYMTAB};
        
        assert!(section.shtype == SHT_SYMTAB);
        unsafe {
            slice_pod(self.section_data(section), 0, section.size as usize / size_of::<Symbol>())
        }
    }

    // Enumerate all the symbols in the file
    pub fn for_each_symbol<F: FnMut(&'s Symbol)>(&self, mut func: F) {
        use super::{SHT_SYMTAB};
        
        for sym in self.section_headers().iter().filter(|s| s.shtype == SHT_SYMTAB).flat_map(|s| self.section_symbols(s).iter()) {
            func(sym);
        }
    }

    /// Create a slice of the section headers.
    pub fn section_headers(&self) -> &'s [SectionHeader] {
        let correct_header_size = self.header.shentsize as usize == size_of::<SectionHeader>();
        let sheader_region_size = self.header.shoff as usize + self.header.shnum as usize * self.header.shentsize as usize;
        let big_enough_region = self.region.len() >= sheader_region_size;

        if self.header.shoff == 0 || !correct_header_size || !big_enough_region {
            return &[];
        }

        unsafe {
            slice_pod(self.region, self.header.shoff as usize, self.header.shnum as usize)
        }
    }

    /// Can we load the binary on our platform?
    // TODO Move this to platform specific.
    fn can_load(&self) -> bool {
        use super::{ELFCLASS64, EV_CURRENT, ELFDATA2LSB, ELFOSABI_SYSV, ELFOSABI_LINUX, ET_EXEC, ET_DYN, EM_X86_64};
        
        let correct_class = {self.header.ident.class} == ELFCLASS64;
        let correct_elfversion = {self.header.ident.version} == EV_CURRENT;
        let correct_data = {self.header.ident.data} == ELFDATA2LSB;
        let correct_osabi = {self.header.ident.osabi} == ELFOSABI_SYSV || {self.header.ident.osabi} == ELFOSABI_LINUX;
        let correct_type = {self.header.elftype} == ET_EXEC || {self.header.elftype} == ET_DYN;
        let correct_machine = {self.header.machine} == EM_X86_64;

        correct_class && correct_data && correct_elfversion && correct_machine && correct_osabi && correct_type
    }
}
