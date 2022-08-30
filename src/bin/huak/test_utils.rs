use std::{
    env::current_dir,
    fs::{copy, create_dir_all, File},
    path::{Path, PathBuf},
};
use zip::ZipArchive;
/// unzips a project from a given zip file into the provided directory.
/// this command only works the CURRENT DIRECTORY is unchchanged.\
/// resource_archive: PathBuf - directory, relative to resources.
pub fn create_py_project_sample(resource_archive: &PathBuf, target_directory: &PathBuf) -> bool {
    if !Path::new(resource_archive).is_file() {
        println!("resource archive does not exist");
    }

    if !Path::new(target_directory).is_dir() {
        println!(
            "target_directory {} does not exist",
            target_directory.as_os_str().to_str().unwrap()
        );
    }

    let file_name = resource_archive.file_name().unwrap().to_str().unwrap();
    let target_archive =
        target_directory.as_os_str().to_str().unwrap().to_owned() + "/" + file_name;

    println!("{}", current_dir().unwrap().as_os_str().to_str().unwrap());
    println!("{}", target_archive);
    copy(resource_archive, &target_archive).unwrap();

    let f = File::open(&target_archive).unwrap();

    let mut archive = ZipArchive::new(f).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => get_path(&target_directory, path),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    true
}

fn get_path(target_directory: &PathBuf, file_path: &Path) -> PathBuf {
    let target_directory_string = target_directory.as_os_str().to_str().unwrap().to_owned()
        + "/"
        + file_path.to_owned().as_os_str().to_str().unwrap();

    PathBuf::from(target_directory_string)
}
