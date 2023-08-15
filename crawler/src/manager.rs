use std::{ fs::{ File, create_dir, remove_dir_all }, path::PathBuf, env::current_dir, io::Read };
use crate::utils::get_current_working_dir;
use std::io::{ Write, Seek };
use std::iter::Iterator;
use walkdir::{ WalkDir, DirEntry };
use zip::write::FileOptions;
use std::path::Path;

pub fn zip_version(repository: &String, version: &String) {
    println!("Zipping {}/{}", repository, version);
    let zipped: PathBuf = get_current_working_dir().unwrap().join("zipped");
    if !zipped.exists() {
        create_dir(&zipped).unwrap();
    }

    // we do this in case some repositories are like name/subpath (e.g. @openzeppelin/contracts)
    let source_name: &str = repository.split("/").collect::<Vec<&str>>()[0];

    let final_zip: PathBuf = zipped.join(format!("{}~{}.zip", &source_name, version));
    let path: &Path = Path::new(&final_zip);
    let file: File = File::create(&path).unwrap();

    let to_zip: String = format!("node_modules/");
    let walkdir: WalkDir = WalkDir::new(&to_zip);
    let it: walkdir::IntoIter = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), &to_zip, file, zip::CompressionMethod::Bzip2).unwrap();

    // removing node modules after zipping
    remove_dir_all(get_current_working_dir().unwrap().join("node_modules")).unwrap();
}

// simple zip directory that walks through a directory and zips it by adding every file to the zip archive
fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod
) -> zip::result::ZipResult<()>
    where T: Write + Seek
{
    let mut zip: zip::ZipWriter<T> = zip::ZipWriter::new(writer);
    let options: FileOptions = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer: Vec<u8> = Vec::new();
    for entry in it {
        let path: &Path = entry.path();
        let name: &Path = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {:?} as {:?} ...", path, name);
            zip.start_file::<&str>(name.to_str().unwrap(), options)?;
            let mut f: File = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory(name.to_str().unwrap(), options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

pub fn push_to_repository(repository: &String, version: &String) {}
