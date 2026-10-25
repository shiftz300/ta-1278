use super::*;
use std::fs;
fn setup(name: &str, ro: bool) -> (Vfs, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("ta1278-vfs-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let mut vfs = Vfs::default();
    vfs.mount(
        "card",
        "存储卡",
        Box::new(Directory::new(&root).unwrap()),
        ro,
    )
    .unwrap();
    (vfs, root)
}
#[test]
fn browse_write_and_remount() {
    let (v, root) = setup("rw", false);
    v.mkdir("/card/Music").unwrap();
    fs::write(root.join("Music/test.wav"), b"audio").unwrap();
    assert_eq!(v.list("/").unwrap()[0].path, "/card");
    assert!(v.list("/card").unwrap()[0].directory);
    v.rename("/card/Music/test.wav", "/card/Music/renamed.wav")
        .unwrap();
    assert!(v.remove("/card/Music").is_err());
    assert!(v.remove("/card").is_err());
    let (mut v, _) = setup("rw", true);
    let mut bytes = vec![];
    v.open("/card/Music/renamed.wav")
        .unwrap()
        .reader
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"audio");
    assert!(v.remove("/card/Music/renamed.wav").is_err());
    assert!(v.mkdir("/card/new").is_err());
    assert!(
        v.rename("/card/Music/renamed.wav", "/card/Music/new.wav")
            .is_err()
    );
    assert!(v.unmount("card"));
    assert!(v.open("/card/Music/renamed.wav").is_err());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn traversal_and_overwrite_are_rejected() {
    let (v, root) = setup("bounds", false);
    for path in [
        "/card/../escape",
        "/card//x",
        "card/x",
        "/card/./x",
        "/card/a\\b",
    ] {
        assert!(v.mkdir(path).is_err(), "{path}");
    }
    fs::write(root.join("one"), b"1").unwrap();
    fs::write(root.join("two"), b"2").unwrap();
    assert!(v.rename("/card/one", "/card/two").is_err());
    assert_eq!(fs::read(root.join("two")).unwrap(), b"2");
    fs::remove_dir_all(root).unwrap();
}
#[cfg(unix)]
#[test]
fn links_cannot_expose_host_files() {
    let (v, root) = setup("links", false);
    std::os::unix::fs::symlink(std::env::temp_dir(), root.join("outside")).unwrap();
    assert!(v.list("/card").unwrap().is_empty());
    assert!(v.list("/card/outside").is_err());
    assert!(v.open("/card/outside/data").is_err());
    assert!(v.mkdir("/card/outside/data").is_err());
    fs::remove_dir_all(root).unwrap();
}
