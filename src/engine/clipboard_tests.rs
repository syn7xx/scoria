use super::*;

#[test]
fn choose_content_prefers_arboard_text_over_pb_text() {
    let ar = Some(Content::Text("from-arboard".into()));
    let pb = Some("from-pb".into());
    let got = choose_clipboard_content(ar, pb).expect("arboard text should win over pb text");
    match got {
        Content::Text(s) => assert_eq!(s, "from-arboard"),
        Content::Image { .. } => panic!("expected text"),
    }
}

#[test]
fn choose_content_falls_back_to_pb_text() {
    let ar = None;
    let pb = Some("from-pb".into());
    let got = choose_clipboard_content(ar, pb).expect("pb text fallback should work");
    match got {
        Content::Text(s) => assert_eq!(s, "from-pb"),
        Content::Image { .. } => panic!("expected text"),
    }
}

#[test]
fn choose_content_errors_when_empty() {
    let ar = None;
    let pb = None;
    let res = choose_clipboard_content(ar, pb);
    assert!(res.is_err());
    let err = res.err().expect("empty clipboard should return error");
    assert!(err
        .to_string()
        .contains(i18n::err_nothing_to_save_selection()));
}

#[test]
fn choose_content_uses_arboard_image_when_present() {
    let ar = Some(Content::Image {
        data: vec![1, 2, 3, 4],
        ext: "png",
    });
    let pb = Some("some text".into());
    let got = choose_clipboard_content(ar, pb).expect("image from arboard should be selected");
    match got {
        Content::Image { .. } => {}
        Content::Text(_) => panic!("expected image"),
    }
}
