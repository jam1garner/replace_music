#![feature(proc_macro_hygiene)]

use skyline::{hook, install_hook};
use skyline::hooks::{getRegionAddress, Region};
use skyline::libc::{c_void, c_char};
use skyline::logging::hex_dump_ptr;
use smash::hash40;
use std::{ptr, fs, io, path::{Path, PathBuf}, collections::HashMap};

struct StreamFiles(pub HashMap<u64, PathBuf>);

const STREAM_DIR: &str = "rom:/stream";

impl StreamFiles {
    fn new() -> Self {
        let mut instance = Self(HashMap::new());

        let _ = instance.visit_dir(Path::new(STREAM_DIR));

        instance
    }

    fn visit_dir(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let filename = entry.path();
                let real_path = format!("{}/{}", dir.display(), filename.display());
                let path = Path::new(&real_path);
                if path.is_dir() {
                    self.visit_dir(&path)?;
                } else {
                    self.visit_file(path);
                }
            }
        }

        Ok(())
    }

    fn visit_file(&mut self, path: &Path) {
        let game_path = format!("stream:{}", &path.display().to_string()[STREAM_DIR.len()..]);
        let hash = hash40(&game_path);
        self.0.insert(hash, path.to_owned());
    }
}

lazy_static::lazy_static!{
    static ref STREAM_FILES: StreamFiles = StreamFiles::new();
}

static mut LOOKUP_STREAM_HASH_OFFSET: usize = 0x31bf2e0; // default = 7.0.0 offset

// (char *out_path,void *loadedArc,undefined8 *size_out,undefined8 *offset_out, ulonglong hash)
#[hook(offset = LOOKUP_STREAM_HASH_OFFSET)]
fn lookup_by_stream_hash(
    out_path: *mut c_char, loaded_arc: *const c_void, size_out: *mut u64, offset_out: *mut u64, hash: u64
) {
    if let Some(path) = STREAM_FILES.0.get(&hash) {
        let file = fs::File::open(&path).unwrap();
        let metadata = file.metadata().unwrap();
        let size = metadata.len() as u64;

        unsafe {
            *size_out = size;
            *offset_out = 0;
            let string = path.to_str().expect("Paths must be valid unicode");
            println!("Loading '{}'...", string);
            let bytes = string.as_bytes();
            ptr::copy_nonoverlapping(
                bytes.as_ptr(), out_path, bytes.len()
            );
            *out_path.offset(bytes.len() as _) = 0u8;
        }
        hex_dump_ptr(out_path);
    } else {
        original!()(out_path, loaded_arc, size_out, offset_out, hash);
    }
}

static SEARCH_CODE: &[u8] = &[
    0x29, 0x58, 0x40, 0xf9, //    ldr        x9,[loadedArc, #0xb0]
    0x28, 0x60, 0x40, 0xf9, //    ldr        x8,[loadedArc, #0xc0]
    0x2a, 0x05, 0x40, 0xb9, //    ldr        w10,[x9, #0x4]
    0x09, 0x0d, 0x0a, 0x8b, //    add        x9,x8,x10, LSL #0x3
    0xaa, 0x01, 0x00, 0x34, //    cbz        w10,LAB_71031bf324
    0x5f, 0x01, 0x00, 0xf1, //    cmp        x10,#0x0
];

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

#[skyline::main(name = "replace_music")]
pub fn main() {
    lazy_static::initialize(&STREAM_FILES);

    unsafe {
        let text_ptr = getRegionAddress(Region::Text) as *const u8;
        let text_size = (getRegionAddress(Region::Rodata) as usize) - (text_ptr as usize);
        let text = std::slice::from_raw_parts(text_ptr, text_size);
        if let Some(offset) = find_subsequence(text, SEARCH_CODE) {
            LOOKUP_STREAM_HASH_OFFSET = offset
        } else {
            println!("Error: no offset found. Defaulting to 7.0.0 offset. This likely won't work.");
        }
    }
    
    install_hook!(lookup_by_stream_hash);
    println!("Music replacement mod installed");
}
