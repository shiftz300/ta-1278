use super::*;
use std::fs;
struct BrokenSource(Directory);
impl Volume for BrokenSource {
    fn list(&self, p: &str) -> io::Result<Vec<Entry>> {
        self.0.list(p)
    }
    fn open(&self, p: &str) -> io::Result<OpenFile> {
        if p.ends_with("bad") {
            Err(io::Error::other("Read failure"))
        } else {
            self.0.open(p)
        }
    }
    fn mkdir(&self, p: &str) -> io::Result<()> {
        self.0.mkdir(p)
    }
    fn remove(&self, p: &str) -> io::Result<()> {
        self.0.remove(p)
    }
    fn rename(&self, a: &str, b: &str) -> io::Result<()> {
        self.0.rename(a, b)
    }
}
#[test]
fn read_failure_rolls_back_copied_children_and_keeps_every_source() {
    let (mut v, root) = setup("rollback", false);
    let source = root.join("source");
    fs::create_dir_all(source.join("folder")).unwrap();
    fs::write(source.join("folder/a"), b"one").unwrap();
    fs::write(source.join("folder/bad"), b"two").unwrap();
    v.mount(
        "source",
        "source",
        Box::new(BrokenSource(Directory::new(&source).unwrap())),
        false,
    )
    .unwrap();
    assert!(v.transfer("/source/folder", "/card/copied", true).is_err());
    assert!(!root.join("copied").exists());
    assert_eq!(fs::read(source.join("folder/a")).unwrap(), b"one");
    assert_eq!(fs::read(source.join("folder/bad")).unwrap(), b"two");
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn recursive_copy_cross_volume_move_and_conflict_preserve_sources() {
    let (mut v, root) = setup("transfer", false);
    let other = root.join("other");
    fs::create_dir(&other).unwrap();
    v.mount(
        "phone",
        "手机存储",
        Box::new(Directory::new(&other).unwrap()),
        false,
    )
    .unwrap();
    fs::create_dir_all(root.join("Music/sub")).unwrap();
    fs::write(root.join("Music/sub/tone.wav"), b"audio").unwrap();
    v.transfer("/card/Music", "/phone/Songs", false).unwrap();
    assert_eq!(
        fs::read(other.join("Songs/sub/tone.wav")).unwrap(),
        b"audio"
    );
    assert!(root.join("Music/sub/tone.wav").is_file());
    assert!(v.transfer("/card/Music", "/phone/Songs", true).is_err());
    assert!(root.join("Music/sub/tone.wav").is_file());
    assert!(
        v.transfer("/card/Music", "/card/Music/nested", false)
            .is_err()
    );
    v.transfer("/card/Music", "/phone/Moved", true).unwrap();
    assert!(!root.join("Music").exists());
    assert!(other.join("Moved/sub/tone.wav").is_file());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn transfers_respect_read_only_source_and_destination() {
    let (mut v, root) = setup("transfer-ro", true);
    let target = root.join("dest");
    fs::create_dir(&target).unwrap();
    fs::write(root.join("source"), b"x").unwrap();
    v.mount(
        "phone",
        "phone",
        Box::new(Directory::new(&target).unwrap()),
        false,
    )
    .unwrap();
    assert!(v.transfer("/card/source", "/phone/one", true).is_err());
    v.transfer("/card/source", "/phone/one", false).unwrap();
    assert!(v.transfer("/phone/one", "/card/two", false).is_err());
    assert!(v.transfer("/card", "/phone/root", false).is_err());
    assert_eq!(fs::read(root.join("source")).unwrap(), b"x");
    fs::remove_dir_all(root).unwrap();
}
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
