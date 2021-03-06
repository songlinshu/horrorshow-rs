#![cfg(feature = "alloc")]

extern crate alloc;
#[macro_use]
extern crate horrorshow;

use horrorshow::{Raw, Template};

#[test]
fn test_prim() {
    assert_eq!(
        html! {
            : 1.01;
            : 2i32;
            : 3usize;
            : 'c'
        }
        .into_string()
        .unwrap(),
        "1.0123c"
    );
}

#[test]
fn test_hyphen() {
    assert_eq!(
        html! {
            foo-bar {
                : "foo"
            }
        }
        .into_string()
        .unwrap(),
        "<foo-bar>foo</foo-bar>"
    );
    assert_eq!(
        html! {
            foo-bar : "foo"
        }
        .into_string()
        .unwrap(),
        "<foo-bar>foo</foo-bar>"
    );
    assert_eq!(
        html! {
            foo-bar;
        }
        .into_string()
        .unwrap(),
        "<foo-bar></foo-bar>"
    );
}

#[test]
fn test_reentrant() {
    assert_eq!(
        &html! {
            p : format_args!("{}", html! { a(href="abcde") }.into_string().unwrap())
        }
        .into_string()
        .unwrap(),
        "<p>&lt;a href=&quot;abcde&quot;&gt;&lt;/a&gt;</p>"
    );

    assert_eq!(
        &html! {
            p {
                |tmpl| tmpl << (html! { a(href="abcde") }).into_string().unwrap();
            }
        }
        .into_string()
        .unwrap(),
        "<p>&lt;a href=&quot;abcde&quot;&gt;&lt;/a&gt;</p>"
    );

    assert_eq!(
        &html! {
            p {
                : Raw(html! { a(href="abcde") }.into_string().unwrap());
            }
        }
        .into_string()
        .unwrap(),
        "<p><a href=\"abcde\"></a></p>"
    );
}

#[test]
fn test_option() {
    assert_eq!(
        html! {
            tag : Some("testing")
        }
        .into_string()
        .unwrap(),
        "<tag>testing</tag>"
    );

    assert_eq!(
        html! {
            tag : None::<&str>
        }
        .into_string()
        .unwrap(),
        "<tag></tag>"
    );
}

#[test]
fn test_into_string_by_ref() {
    let r = html! {
        |tmpl| tmpl << "test";
    };
    assert_eq!((&r).into_string().unwrap(), "test");
    assert_eq!((&r).into_string().unwrap(), "test");
}

#[test]
fn test_embed_twice() {
    let r = html! {
        |tmpl| {
            let sub = html! { : "abcde" };
            &mut *tmpl << &sub;
            &mut *tmpl << &sub;
        }
    };
    assert_eq!(r.into_string().unwrap(), "abcdeabcde");
}

#[test]
fn test_display() {
    use alloc::fmt::Write;
    let r = html! {
        |tmpl| tmpl << "test";
    };
    let mut s = String::new();
    write!(s, "{}", r).unwrap();
    assert_eq!(&s, "test");
}
