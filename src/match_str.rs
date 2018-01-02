pub fn match_str<'a>(data: &'a str, pattern: &'a str) -> Option<&'a str> {
    let pattern_len = pattern.len();

    if data.len() >= pattern_len {
        let (head, body) = data.split_at(pattern_len);
        if head == pattern {
            return Some(body);
        }
    }

    None
}

#[test]
fn match_str_test() {
    let aaa = "abcdef";

    assert_eq!(match_str(aaa, ""), Some("abcdef"));
    assert_eq!(match_str(aaa, "abc"), Some("def"));
    assert_eq!(match_str(aaa, "abcde"), Some("f"));
    assert_eq!(match_str(aaa, "abcdef"), Some(""));
    assert_eq!(match_str(aaa, "abd"), None);
    assert_eq!(match_str(aaa, "abdeffffff"), None);
    assert_eq!(match_str(aaa, "ffff"), None);
}
