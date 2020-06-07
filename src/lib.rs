#![feature(proc_macro_hygiene)]

use skyline::{hook, install_hook};
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

// (char *out_path,void *loadedArc,undefined8 *size_out,undefined8 *offset_out, ulonglong hash)
#[hook(offset = 0x31bf2e0)]
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

#[skyline::main(name = "replace_music")]
pub fn main() {
    lazy_static::initialize(&STREAM_FILES);
    install_hook!(lookup_by_stream_hash);
    println!("Music replacement mod installed");
}
