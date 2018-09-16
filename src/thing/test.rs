use nature_common::*;

#[test]
fn standardize_empty() {
    println!("----------------- standardize_empty --------------------");
    let mut key = String::new();
    let rtn = Thing::key_standardize(&mut key);
    if let Err(NatureError::VerifyError(x)) = rtn {
        assert_eq!(x, "key length can't be zero");
    } else {
        panic!("should get error")
    }

    let mut key = "/".to_string();
    let rtn = Thing::key_standardize(&mut key);
    if let Err(NatureError::VerifyError(x)) = rtn {
        assert_eq!(x, "key length can't be zero");
    } else {
        panic!("should get error")
    }
}

/// also test for removing last separator and Business prefix
#[test]
fn standardize_no_separator_at_beginning() {
    println!("----------------- standardize_no_separator_at_beginning --------------------");
    let mut key = "a/b/c/".to_string();
    let _rtn = Thing::key_standardize(&mut key);
    assert_eq!(key, "/a/b/c");
}
