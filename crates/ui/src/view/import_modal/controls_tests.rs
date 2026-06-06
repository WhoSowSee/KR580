use super::controls::shorten_middle;

#[test]
fn file_label_shortens_from_middle() {
    let out = shorten_middle("C:\\Users\\Long\\Folder\\import-file.xlsx", 18);
    assert!(out.chars().count() <= 18);
    assert!(out.contains('…'));
}
