use super::*;
#[cfg(target_os = "windows")]
#[test]
fn windows_names_cannot_open_devices_or_alternate_streams() {
    for name in [
        "CON", "nul.txt", "COM1", "lpt9.log", "a:stream", "name.", "name ", "a?b", "a*b",
    ] {
        assert!(!valid_name(name), "{name}");
    }
    assert!(valid_name("Music 2026"));
    assert!(valid_name("COM10.txt"));
}
fn setup(name: &str) -> (Folder, PathBuf) {
    let root = std::env::temp_dir().join(format!("ta1278-folder-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    (Folder::new(&root, false).unwrap(), root)
}
#[test]
fn direct_files_copy_move_and_read_only() {
    let (folder, root) = setup("operations");
    folder.mkdir("/Music").unwrap();
    fs::write(root.join("Music/tone.wav"), b"audio").unwrap();
    assert_eq!(folder.list("/").unwrap()[0].path, "/Music");
    folder.transfer("/Music", "/Copy", false).unwrap();
    assert_eq!(fs::read(root.join("Copy/tone.wav")).unwrap(), b"audio");
    assert!(folder.transfer("/Music", "/Copy", false).is_err());
    assert!(folder.transfer("/Music", "/Music/inside", false).is_err());
    folder.transfer("/Copy", "/Moved", true).unwrap();
    assert!(!root.join("Copy").exists());
    assert!(folder.remove("/Music").is_err());
    assert!(folder.remove("/").is_err());
    let readonly = Folder::new(&root, true).unwrap();
    assert!(readonly.open("/Music/tone.wav").is_ok());
    assert!(readonly.mkdir("/New").is_err());
    assert!(readonly.rename("/Music", "/New").is_err());
    assert!(readonly.transfer("/Music", "/New", false).is_err());
    assert!(readonly.remove("/Music/tone.wav").is_err());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn reject_traversal_overwrite_and_links() {
    let (folder, root) = setup("paths");
    for path in ["/../outside", "/a//b", "/./a", "relative", "/a\\b"] {
        assert!(folder.mkdir(path).is_err(), "{path}");
    }
    fs::write(root.join("one"), b"1").unwrap();
    fs::write(root.join("two"), b"2").unwrap();
    assert!(folder.rename("/one", "/two").is_err());
    assert_eq!(fs::read(root.join("two")).unwrap(), b"2");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(std::env::temp_dir(), root.join("outside")).unwrap();
        assert!(folder.list("/outside").is_err());
        assert!(folder.mkdir("/outside/new").is_err());
        assert!(
            folder
                .list("/")
                .unwrap()
                .iter()
                .all(|e| e.name != "outside")
        );
    }
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn batch_delete_preflights_all_entries_and_never_recurses_or_ignores_readonly() {
    let (folder, root) = setup("delete-batch");
    fs::write(root.join("a"), b"keep").unwrap();
    fs::create_dir(root.join("nonempty")).unwrap();
    fs::write(root.join("nonempty/data"), b"keep").unwrap();
    let paths = vec!["/a".into(), "/nonempty".into()];
    let (done, failed) = folder.remove_many(&paths);
    assert!(done.is_empty() && !failed.is_empty());
    assert!(root.join("a").exists());
    let ro = Folder::new(&root, true).unwrap();
    assert!(ro.remove_many(&["/a".into()]).0.is_empty());
    assert!(root.join("a").exists());
    fs::create_dir(root.join("empty")).unwrap();
    let (done, failed) = folder.remove_many(&["/a".into(), "/empty".into()]);
    assert_eq!(done.len(), 2);
    assert!(failed.is_empty());
    assert_eq!(fs::read(root.join("nonempty/data")).unwrap(), b"keep");
    fs::remove_dir_all(root).unwrap();
}
