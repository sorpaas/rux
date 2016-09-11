use core::fmt;

mod loader;

pub use self::loader::{ElfBinary};

/// ELF magic number
pub const ELF_MAGIC: &'static [u8] = &[0x7f, 'E' as u8, 'L' as u8, 'F' as u8];

/// Represents the ELF file class (32-bit vs 64-bit)
#[derive(Copy, Clone, PartialEq)]
pub struct Class(pub u8);

/// Invalid ELF file class
pub const ELFCLASSNONE : Class = Class(0);
/// 32-bit ELF file
pub const ELFCLASS32 : Class = Class(1);
/// 64-bit ELF file
pub const ELFCLASS64 : Class = Class(2);

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            ELFCLASSNONE => "Invalid",
            ELFCLASS32 => "32-bit",
            ELFCLASS64 => "64-bit",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

/// Represents the ELF file data format (little-endian vs big-endian)
#[derive(Copy, Clone, PartialEq)]
pub struct Data(pub u8);

/// Invalid ELF data format
pub const ELFDATANONE : Data = Data(0);
/// little-endian ELF file
pub const ELFDATA2LSB : Data = Data(1);
/// big-endian ELF file
pub const ELFDATA2MSB : Data = Data(2);

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            ELFDATANONE => "Invalid",
            ELFDATA2LSB => "2's complement, little endian",
            ELFDATA2MSB => "2's complement, big endian",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

/// Represents the ELF file version
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Version(pub u8);

/// Invalid version
pub const EV_NONE : Version = Version(0);
/// Current version
pub const EV_CURRENT : Version = Version(1);

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            EV_NONE => "Invalid",
            EV_CURRENT => "1 (Current)",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

/// Represents the ELF file OS ABI
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct OSABI(pub u8);

/// Defaults to Unix System V
pub const ELFOSABI_NONE : OSABI = OSABI(0);
/// Unix System V
pub const ELFOSABI_SYSV : OSABI = OSABI(0);
/// HP-UX
pub const ELFOSABI_HPUX : OSABI = OSABI(1);
/// NetBSD
pub const ELFOSABI_NETBSD : OSABI = OSABI(2);
/// Linux with GNU extensions
pub const ELFOSABI_LINUX : OSABI = OSABI(3);
/// Solaris
pub const ELFOSABI_SOLARIS : OSABI = OSABI(6);
/// AIX
pub const ELFOSABI_AIX : OSABI = OSABI(7);
/// SGI Irix
pub const ELFOSABI_IRIX : OSABI = OSABI(8);
/// FreeBSD
pub const ELFOSABI_FREEBSD : OSABI = OSABI(9);
/// Compaq TRU64 UNIX
pub const ELFOSABI_TRU64 : OSABI = OSABI(10);
/// Novell Modesto
pub const ELFOSABI_MODESTO : OSABI = OSABI(11);
/// OpenBSD
pub const ELFOSABI_OPENBSD : OSABI = OSABI(12);

impl fmt::Debug for OSABI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for OSABI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            ELFOSABI_SYSV => "UNIX System V",
            ELFOSABI_HPUX => "HP-UX",
            ELFOSABI_NETBSD => "NetBSD",
            ELFOSABI_LINUX => "Linux with GNU extensions",
            ELFOSABI_SOLARIS => "Solaris",
            ELFOSABI_AIX => "AIX",
            ELFOSABI_IRIX => "SGI Irix",
            ELFOSABI_FREEBSD => "FreeBSD",
            ELFOSABI_TRU64 => "Compaq TRU64 UNIX",
            ELFOSABI_MODESTO => "Novell Modesto",
            ELFOSABI_OPENBSD => "OpenBSD",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

/// Represents the ELF file type (object, executable, shared lib, core)
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Type(pub u16);
/// No file type
pub const ET_NONE : Type = Type(0);
/// Relocatable object file
pub const ET_REL : Type = Type(1);
/// Executable file
pub const ET_EXEC : Type = Type(2);
/// Shared library
pub const ET_DYN : Type = Type(3);
/// Core file
pub const ET_CORE : Type = Type(4);

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            ET_NONE => "No file type",
            ET_REL => "Relocatable file",
            ET_EXEC => "Executable file",
            ET_DYN => "Shared object file",
            ET_CORE => "Core file",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

/// Represents the ELF file machine architecture
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Machine(pub u16);
pub const EM_NONE : Machine = Machine(0);
pub const EM_M32 : Machine = Machine(1);
pub const EM_SPARC : Machine = Machine(2);
pub const EM_386 : Machine = Machine(3);
pub const EM_68K : Machine = Machine(4);
pub const EM_88K : Machine = Machine(5);
pub const EM_860 : Machine = Machine(7);
pub const EM_MIPS : Machine = Machine(8);
pub const EM_S370 : Machine = Machine(9);
pub const EM_MIPS_RS3_LE : Machine = Machine(10);
pub const EM_PARISC : Machine = Machine(15);
pub const EM_VPP500 : Machine = Machine(17);
pub const EM_SPARC32PLUS : Machine = Machine(18);
pub const EM_960 : Machine = Machine(19);
pub const EM_PPC : Machine = Machine(20);
pub const EM_PPC64 : Machine = Machine(21);
pub const EM_S390 : Machine = Machine(22);
pub const EM_V800 : Machine = Machine(36);
pub const EM_FR20 : Machine = Machine(37);
pub const EM_RH32 : Machine = Machine(38);
pub const EM_RCE : Machine = Machine(39);
pub const EM_ARM : Machine = Machine(40);
pub const EM_FAKE_ALPHA : Machine = Machine(41);
pub const EM_SH : Machine = Machine(42);
pub const EM_SPARCV9 : Machine = Machine(43);
pub const EM_TRICORE : Machine = Machine(44);
pub const EM_ARC : Machine = Machine(45);
pub const EM_H8_300 : Machine = Machine(46);
pub const EM_H8_300H : Machine = Machine(47);
pub const EM_H8S : Machine = Machine(48);
pub const EM_H8_500 : Machine = Machine(49);
pub const EM_IA_64 : Machine = Machine(50);
pub const EM_MIPS_X : Machine = Machine(51);
pub const EM_COLDFIRE : Machine = Machine(52);
pub const EM_68HC12 : Machine = Machine(53);
pub const EM_MMA : Machine = Machine(54);
pub const EM_PCP : Machine = Machine(55);
pub const EM_NCPU : Machine = Machine(56);
pub const EM_NDR1 : Machine = Machine(57);
pub const EM_STARCORE : Machine = Machine(58);
pub const EM_ME16 : Machine = Machine(59);
pub const EM_ST100 : Machine = Machine(60);
pub const EM_TINYJ : Machine = Machine(61);
pub const EM_X86_64 : Machine = Machine(62);
pub const EM_PDSP : Machine = Machine(63);
pub const EM_FX66 : Machine = Machine(66);
pub const EM_ST9PLUS : Machine = Machine(67);
pub const EM_ST7 : Machine = Machine(68);
pub const EM_68HC16 : Machine = Machine(69);
pub const EM_68HC11 : Machine = Machine(70);
pub const EM_68HC08 : Machine = Machine(71);
pub const EM_68HC05 : Machine = Machine(72);
pub const EM_SVX : Machine = Machine(73);
pub const EM_ST19 : Machine = Machine(74);
pub const EM_VAX : Machine = Machine(75);
pub const EM_CRIS : Machine = Machine(76);
pub const EM_JAVELIN : Machine = Machine(77);
pub const EM_FIREPATH : Machine = Machine(78);
pub const EM_ZSP : Machine = Machine(79);
pub const EM_MMIX : Machine = Machine(80);
pub const EM_HUANY : Machine = Machine(81);
pub const EM_PRISM : Machine = Machine(82);
pub const EM_AVR : Machine = Machine(83);
pub const EM_FR30 : Machine = Machine(84);
pub const EM_D10V : Machine = Machine(85);
pub const EM_D30V : Machine = Machine(86);
pub const EM_V850 : Machine = Machine(87);
pub const EM_M32R : Machine = Machine(88);
pub const EM_MN10300 : Machine = Machine(89);
pub const EM_MN10200 : Machine = Machine(90);
pub const EM_PJ : Machine = Machine(91);
pub const EM_OPENRISC : Machine = Machine(92);
pub const EM_ARC_A5 : Machine = Machine(93);
pub const EM_XTENSA : Machine = Machine(94);
pub const EM_AARCH64 : Machine = Machine(183);
pub const EM_TILEPRO : Machine = Machine(188);
pub const EM_MICROBLAZE : Machine = Machine(189);
pub const EM_TILEGX : Machine = Machine(191);

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            EM_NONE => "No machine",
            EM_M32 => "AT&T WE 32100",
            EM_SPARC => "SUN SPARC",
            EM_386 => "Intel 80386",
            EM_68K => "Motorola m68k family",
            EM_88K => "Motorola m88k family",
            EM_860 => "Intel 80860",
            EM_MIPS => "MIPS R3000 big-endian",
            EM_S370 => "IBM System/370",
            EM_MIPS_RS3_LE => "MIPS R3000 little-endian",
            EM_PARISC => "HPPA",
            EM_VPP500 => "Fujitsu VPP500",
            EM_SPARC32PLUS => "Sun's 'v8plus'",
            EM_960 => "Intel 80960",
            EM_PPC => "PowerPC",
            EM_PPC64 => "PowerPC 64-bit",
            EM_S390 => "IBM S390",
            EM_V800 => "NEC V800 series",
            EM_FR20 => "Fujitsu FR20",
            EM_RH32 => "TRW RH-32",
            EM_RCE => "Motorola RCE",
            EM_ARM => "ARM",
            EM_FAKE_ALPHA => "Digital Alpha",
            EM_SH => "Hitachi SH",
            EM_SPARCV9 => "SPARC v9 64-bit",
            EM_TRICORE => "Siemens Tricore",
            EM_ARC => "Argonaut RISC Core",
            EM_H8_300 => "Hitachi H8/300",
            EM_H8_300H => "Hitachi H8/300H",
            EM_H8S => "Hitachi H8S",
            EM_H8_500 => "Hitachi H8/500",
            EM_IA_64 => "Intel Merced",
            EM_MIPS_X => "Stanford MIPS-X",
            EM_COLDFIRE => "Motorola Coldfire",
            EM_68HC12 => "Motorola M68HC12",
            EM_MMA => "Fujitsu MMA Multimedia Accelerato",
            EM_PCP => "Siemens PCP",
            EM_NCPU => "Sony nCPU embeeded RISC",
            EM_NDR1 => "Denso NDR1 microprocessor",
            EM_STARCORE => "Motorola Start*Core processor",
            EM_ME16 => "Toyota ME16 processor",
            EM_ST100 => "STMicroelectronic ST100 processor",
            EM_TINYJ => "Advanced Logic Corp. Tinyj emb.fa",
            EM_X86_64 => "AMD x86-64 architecture",
            EM_PDSP => "Sony DSP Processor",
            EM_FX66 => "Siemens FX66 microcontroller",
            EM_ST9PLUS => "STMicroelectronics ST9+ 8/16 mc",
            EM_ST7 => "STmicroelectronics ST7 8 bit mc",
            EM_68HC16 => "Motorola MC68HC16 microcontroller",
            EM_68HC11 => "Motorola MC68HC11 microcontroller",
            EM_68HC08 => "Motorola MC68HC08 microcontroller",
            EM_68HC05 => "Motorola MC68HC05 microcontroller",
            EM_SVX => "Silicon Graphics SVx",
            EM_ST19 => "STMicroelectronics ST19 8 bit mc",
            EM_VAX => "Digital VAX",
            EM_CRIS => "Axis Communications 32-bit embedded processor",
            EM_JAVELIN => "Infineon Technologies 32-bit embedded processor",
            EM_FIREPATH => "Element 14 64-bit DSP Processor",
            EM_ZSP => "LSI Logic 16-bit DSP Processor",
            EM_MMIX => "Donald Knuth's educational 64-bit processor",
            EM_HUANY => "Harvard University machine-independent object files",
            EM_PRISM => "SiTera Prism",
            EM_AVR => "Atmel AVR 8-bit microcontroller",
            EM_FR30 => "Fujitsu FR30",
            EM_D10V => "Mitsubishi D10V",
            EM_D30V => "Mitsubishi D30V",
            EM_V850 => "NEC v850",
            EM_M32R => "Mitsubishi M32R",
            EM_MN10300 => "Matsushita MN10300",
            EM_MN10200 => "Matsushita MN10200",
            EM_PJ => "picoJava",
            EM_OPENRISC => "OpenRISC 32-bit embedded processor",
            EM_ARC_A5 => "ARC Cores Tangent-A5",
            EM_XTENSA => "Tensilica Xtensa Architecture",
            EM_AARCH64 => "ARM AARCH64",
            EM_TILEPRO => "Tilera TILEPro",
            EM_MICROBLAZE => "Xilinx MicroBlaze",
            EM_TILEGX => "Tilera TILE-Gx",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

/// First 16 bytes of the ELF file header.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct ElfIdent {
    /// Must have value [0x7f, 'E', 'L', 'F'].
    pub magic: [u8; 4],

    /// 32-bit vs 64-bit
    pub class:      Class,
    /// little vs big endian
    pub data:       Data,
    /// elf version
    pub version:    Version,
    /// OS ABI
    pub osabi:      OSABI,
    /// Version of the OS ABI
    pub abiversion: u8,
    // Reserved (should be zero).
    pub padding: [u8; 7],
}

impl fmt::Display for ElfIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let valid_magic = match self.magic == ELF_MAGIC {
                                    true => "valid magic",
                                    _ => "invalid magic"
                                };
        write!(f, "ElfIdent: {} {} {} {} {}", valid_magic, self.class, self.data, self.version, self.osabi)
    }
}

/// Encapsulates the contents of the ELF File Header
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FileHeader {
    pub ident: ElfIdent,
    /// ELF file type
    pub elftype:    Type,
    /// Target machine architecture
    pub machine:    Machine,
    /// ELF version
    pub version:    u32,
    /// Virtual address of program entry point
    pub entry:      u64,
    /// Start of program headers (bytes into file)
    pub phoff:      u64,
    /// Start of section headers (bytes into file)
    pub shoff:      u64,
    pub flags:      u32,
    /// Size of this header.
    pub ehsize:     u16,
    /// Size of program headers.
    pub phentsize:  u16,
    /// Number of program headers.
    pub phnum:      u16,
    /// Size of section headers.
    pub shentsize:  u16,
    /// Number of section headers.
    pub shnum:      u16,
    /// Section header string table index.
    pub shstrndx:   u16,
}

impl fmt::Display for FileHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileHeader: [{}] is {} for {} in version {} starts at {:x}",
               self.ident, self.elftype, self.machine,
               self.version, self.entry)
    }
}

/// Represents ELF Program Header flags
#[derive(Copy, Clone, PartialEq)]
pub struct ProgFlag(pub u32);

pub const PF_NONE : ProgFlag = ProgFlag(0);
/// Executable program segment
pub const PF_X : ProgFlag = ProgFlag(1);
/// Writable program segment
pub const PF_W : ProgFlag = ProgFlag(2);
/// Readable program segment
pub const PF_R : ProgFlag = ProgFlag(4);

impl fmt::Debug for ProgFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for ProgFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if (self.0 & PF_R.0) != 0 {
            try!(write!(f, "R"));
        } else {
            try!(write!(f, " "));
        }
        if (self.0 & PF_W.0) != 0 {
            try!(write!(f, "W"));
        } else {
            try!(write!(f, " "));
        }
        if (self.0 & PF_X.0) != 0 {
            write!(f, "E")
        } else {
            write!(f, " ")
        }
    }
}

/// Represents ELF Program Header type
#[derive(Copy, Clone, PartialEq)]
pub struct ProgType(pub u32);

/// Program header table entry unused
pub const PT_NULL : ProgType = ProgType(0);
/// Loadable program segment
pub const PT_LOAD : ProgType = ProgType(1);
/// Dynamic linking information
pub const PT_DYNAMIC : ProgType = ProgType(2);
/// Program interpreter
pub const PT_INTERP : ProgType = ProgType(3);
/// Auxiliary information
pub const PT_NOTE : ProgType = ProgType(4);
/// Unused
pub const PT_SHLIB : ProgType = ProgType(5);
/// The program header table
pub const PT_PHDR : ProgType = ProgType(6);
/// Thread-local storage segment
pub const PT_TLS : ProgType = ProgType(7);
/// GCC .eh_frame_hdr segment
pub const PT_GNU_EH_FRAME : ProgType = ProgType(0x6474e550);
/// Indicates stack executability
pub const PT_GNU_STACK : ProgType = ProgType(0x6474e551);
/// Read-only after relocation
pub const PT_GNU_RELRO : ProgType = ProgType(0x6474e552);

impl fmt::Debug for ProgType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for ProgType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            PT_NULL => "NULL",
            PT_LOAD => "LOAD",
            PT_DYNAMIC => "DYNAMIC",
            PT_INTERP => "INTERP",
            PT_NOTE => "NOTE",
            PT_SHLIB => "SHLIB",
            PT_PHDR => "PHDR",
            PT_TLS => "TLS",
            PT_GNU_EH_FRAME => "GNU_EH_FRAME",
            PT_GNU_STACK => "GNU_STACK",
            PT_GNU_RELRO => "GNU_RELRO",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

/// Encapsulates the contents of an ELF Program Header
///
/// The program header table is an array of program header structures describing
/// the various segments for program execution.
#[derive(Copy, Clone, Debug)]
pub struct ProgramHeader {
    /// Program segment type
    pub progtype: ProgType,
    /// Flags for this segment
    pub flags:    ProgFlag,
    /// Offset into the ELF file where this segment begins
    pub offset:   u64,
    /// Virtual address where this segment should be loaded
    pub vaddr:    usize,
    /// Physical address where this segment should be loaded
    pub paddr:    u64,
    /// Size of this segment in the file
    pub filesz:   u64,
    /// Size of this segment in memory
    pub memsz:    u64,
    /// file and memory alignment
    pub align:    u64,
}

impl fmt::Display for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Program Header: Type: {} Offset: {:#010x} VirtAddr: {:#010x} PhysAddr: {:#010x} FileSize: {:#06x} MemSize: {:#06x} Flags: {} Align: {:#x}",
            self.progtype, self.offset, self.vaddr, self.paddr, self.filesz,
            self.memsz, self.flags, self.align)
    }
}

/// Represens ELF Section type
#[derive(Copy, Clone, PartialEq)]
pub struct SectionType(pub u32);

/// Inactive section with undefined values
pub const SHT_NULL : SectionType = SectionType(0);
/// Information defined by the program, includes executable code and data
pub const SHT_PROGBITS : SectionType = SectionType(1);
/// Section data contains a symbol table
pub const SHT_SYMTAB : SectionType = SectionType(2);
/// Section data contains a string table
pub const SHT_STRTAB : SectionType = SectionType(3);
/// Section data contains relocation entries with explicit addends
pub const SHT_RELA : SectionType = SectionType(4);
/// Section data contains a symbol hash table. Must be present for dynamic linking
pub const SHT_HASH : SectionType = SectionType(5);
/// Section data contains information for dynamic linking
pub const SHT_DYNAMIC : SectionType = SectionType(6);
/// Section data contains information that marks the file in some way
pub const SHT_NOTE : SectionType = SectionType(7);
/// Section data occupies no space in the file but otherwise resembles SHT_PROGBITS
pub const SHT_NOBITS : SectionType = SectionType(8);
/// Section data contains relocation entries without explicit addends
pub const SHT_REL : SectionType = SectionType(9);
/// Section is reserved but has unspecified semantics
pub const SHT_SHLIB : SectionType = SectionType(10);
/// Section data contains a minimal set of dynamic linking symbols
pub const SHT_DYNSYM : SectionType = SectionType(11);
/// Section data contains an array of constructors
pub const SHT_INIT_ARRAY : SectionType = SectionType(14);
/// Section data contains an array of destructors
pub const SHT_FINI_ARRAY : SectionType = SectionType(15);
/// Section data contains an array of pre-constructors
pub const SHT_PREINIT_ARRAY : SectionType = SectionType(16);
/// Section group
pub const SHT_GROUP : SectionType = SectionType(17);
/// Extended symbol table section index
pub const SHT_SYMTAB_SHNDX : SectionType = SectionType(18);
/// Number of reserved SHT_* values
pub const SHT_NUM : SectionType = SectionType(19);
/// Object attributes
pub const SHT_GNU_ATTRIBUTES : SectionType = SectionType(0x6ffffff5);
/// GNU-style hash section
pub const SHT_GNU_HASH : SectionType = SectionType(0x6ffffff6);
/// Pre-link library list
pub const SHT_GNU_LIBLIST : SectionType = SectionType(0x6ffffff7);
/// Version definition section
pub const SHT_GNU_VERDEF : SectionType = SectionType(0x6ffffffd);
/// Version needs section
pub const SHT_GNU_VERNEED : SectionType = SectionType(0x6ffffffe);
/// Version symbol table
pub const SHT_GNU_VERSYM : SectionType = SectionType(0x6fffffff);

impl fmt::Debug for SectionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for SectionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            SHT_NULL => "SHT_NULL",
            SHT_PROGBITS => "SHT_PROGBITS",
            SHT_SYMTAB => "SHT_SYMTAB",
            SHT_STRTAB => "SHT_STRTAB",
            SHT_RELA => "SHT_RELA",
            SHT_HASH => "SHT_HASH",
            SHT_DYNAMIC => "SHT_DYNAMIC",
            SHT_NOTE => "SHT_NOTE",
            SHT_NOBITS => "SHT_NOBITS",
            SHT_REL => "SHT_REL",
            SHT_SHLIB => "SHT_SHLIB",
            SHT_DYNSYM => "SHT_DYNSYM",
            SHT_INIT_ARRAY => "SHT_INIT_ARRAY",
            SHT_FINI_ARRAY => "SHT_FINI_ARRAY",
            SHT_PREINIT_ARRAY => "SHT_PREINIT_ARRAY",
            SHT_GROUP => "SHT_GROUP",
            SHT_SYMTAB_SHNDX => "SHT_SYMTAB_SHNDX",
            SHT_NUM => "SHT_NUM",
            SHT_GNU_ATTRIBUTES => "SHT_GNU_ATTRIBUTES",
            SHT_GNU_HASH => "SHT_GNU_HASH",
            SHT_GNU_LIBLIST => "SHT_GNU_LIBLIST",
            SHT_GNU_VERDEF => "SHT_GNU_VERDEF",
            SHT_GNU_VERNEED => "SHT_GNU_VERNEED",
            SHT_GNU_VERSYM => "SHT_GNU_VERSYM",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

///
/// Wrapper type for SectionFlag
///
#[derive(Copy, Clone, PartialEq)]
pub struct SectionFlag(pub u64);

/// Empty flags
pub const SHF_NONE : SectionFlag = SectionFlag(0);
/// Writable
pub const SHF_WRITE : SectionFlag = SectionFlag(1);
/// Occupies memory during execution
pub const SHF_ALLOC : SectionFlag = SectionFlag(2);
/// Executable
pub const SHF_EXECINSTR : SectionFlag = SectionFlag(4);
/// Might be merged
pub const SHF_MERGE : SectionFlag = SectionFlag(16);
/// Contains nul-terminated strings
pub const SHF_STRINGS : SectionFlag = SectionFlag(32);
/// `sh_info' contains SHT index
pub const SHF_INFO_LINK : SectionFlag = SectionFlag(64);
/// Preserve order after combining
pub const SHF_LINK_ORDER : SectionFlag = SectionFlag(128);
/// Non-standard OS specific handling required
pub const SHF_OS_NONCONFORMING : SectionFlag = SectionFlag(256);
/// Section is member of a group
pub const SHF_GROUP : SectionFlag = SectionFlag(512);
/// Section hold thread-local data
pub const SHF_TLS : SectionFlag = SectionFlag(1024);

impl fmt::Debug for SectionFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for SectionFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

// An offset to a null terminated string in the section string table
#[derive(Copy, Clone)]
pub struct StrOffset(pub u32);

impl fmt::Debug for StrOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl fmt::Display for StrOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

/// Encapsulates the contents of an ELF Section Header
#[derive(Debug)]
pub struct SectionHeader {
    /// Section Name
    pub name:      StrOffset,
    /// Section Type
    pub shtype:    SectionType,
    /// Section Flags
    pub flags:     SectionFlag,
    /// in-memory address where this section is loaded
    pub addr:      u64,
    /// Byte-offset into the file where this section starts
    pub offset:    u64,
    /// Section size in bytes
    pub size:      u64,
    /// Defined by section type
    pub link:      u32,
    /// Defined by section type
    pub info:      u32,
    /// address alignment
    pub addralign: u64,
    /// size of an entry if section data is an array of entries
    pub entsize:   u64,
}

impl fmt::Display for SectionHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Section Header: Name: {} Type: {} Flags: {} Addr: {:#010x} Offset: {:#06x} Size: {:#06x} Link: {} Info: {:#x} AddrAlign: {} EntSize: {}",
            self.name, self.shtype, self.flags, self.addr, self.offset,
            self.size, self.link, self.info, self.addralign, self.entsize)
    }
}

#[derive(Copy, Clone)]
pub struct SymbolType(pub u8);

/// Unspecified symbol type
pub const STT_NOTYPE : SymbolType = SymbolType(0);
/// Data object symbol
pub const STT_OBJECT : SymbolType = SymbolType(1);
/// Code object symbol
pub const STT_FUNC : SymbolType = SymbolType(2);
/// Section symbol
pub const STT_SECTION : SymbolType = SymbolType(3);
/// File name symbol
pub const STT_FILE : SymbolType = SymbolType(4);
/// Common data object symbol
pub const STT_COMMON : SymbolType = SymbolType(5);
/// Thread-local data object symbol
pub const STT_TLS	 : SymbolType = SymbolType(6);
/// Indirect code object symbol
pub const STT_GNU_IFUNC : SymbolType = SymbolType(10);

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            STT_NOTYPE => "unspecified",
            STT_OBJECT => "data object",
            STT_FUNC => "code object",
            STT_SECTION => "section",
            STT_FILE => "file name",
            STT_COMMON => "common data object",
            STT_TLS => "thread-local data object",
            STT_GNU_IFUNC => "indirect code object",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

#[derive(Copy, Clone)]
pub struct SymbolBind(pub u8);

/// Local symbol
pub const STB_LOCAL : SymbolBind = SymbolBind(0);
/// Global symbol
pub const STB_GLOBAL : SymbolBind = SymbolBind(1);
/// Weak symbol
pub const STB_WEAK : SymbolBind = SymbolBind(2);
/// Unique symbol
pub const STB_GNU_UNIQUE : SymbolBind = SymbolBind(10);

impl fmt::Display for SymbolBind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            STB_LOCAL => "local",
            STB_GLOBAL => "global",
            STB_WEAK => "weak",
            STB_GNU_UNIQUE => "unique",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

#[derive(Copy, Clone)]
pub struct SymbolVis(pub u8);

/// Default symbol visibility
pub const STV_DEFAULT : SymbolVis = SymbolVis(0);
/// Processor-specific hidden visibility
pub const STV_INTERNAL : SymbolVis = SymbolVis(1);
/// Hidden visibility
pub const STV_HIDDEN : SymbolVis = SymbolVis(2);
/// Protected visibility
pub const STV_PROTECTED : SymbolVis = SymbolVis(3);

impl fmt::Display for SymbolVis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            STV_DEFAULT => "default",
            STV_INTERNAL => "internal",
            STV_HIDDEN => "hidden",
            STV_PROTECTED => "protected",
            _ => "Unknown",
        };
        write!(f, "{}", str)
    }
}

pub struct Symbol {
    /// Symbol name
    pub name: StrOffset,
    info: u8,
    other: u8,
    section_index: u16,
    /// Symbol value
    pub value: u64,
    /// Symbol size
    pub size: u64,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Symbol: [{}] @ {:#x} size {:#x} in section {}",
               self.name, self.value, self.size, self.section_index)
    }
}

impl Symbol {
    pub fn sym_type(&self) -> SymbolType {
        SymbolType(self.info & 0xf)
    }

    pub fn sym_bind(&self) -> SymbolBind {
        SymbolBind(self.info >> 4)
    }

    pub fn sym_vis(&self) -> SymbolVis {
        SymbolVis(self.other & 0x3)
    }
}
