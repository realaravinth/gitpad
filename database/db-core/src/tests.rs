//! Test utilities
use crate::prelude::*;

/// test email registration implementation
pub async fn email_register_works<T: GistDatabase>(
    db: &T,
    email: &str,
    username: &str,
    password: &str,
    secret: &str,
    username2: &str,
) {
    let _ = db.delete_account(username).await;
    let _ = db.delete_account(username2).await;

    assert!(matches!(
        db.email_login(email).await.err(),
        Some(DBError::AccountNotFound)
    ));

    let mut register_payload = EmailRegisterPayload {
        email,
        username,
        password,
        secret,
    };

    db.email_register(&register_payload).await.unwrap();
    assert!(db.username_exists(username).await.unwrap());
    assert!(db.email_exists(email).await.unwrap());
    assert_eq!(db.get_secret(username).await.unwrap(), secret);
    let login_resp = db.email_login(email).await.unwrap();
    assert_eq!(login_resp.username, username);
    assert_eq!(login_resp.password, password);

    register_payload.secret = email;
    register_payload.username = username2;
    let err = db.email_register(&register_payload).await.err();
    assert!(matches!(err, Some(DBError::DuplicateEmail)));
}

/// test username registration implementation
pub async fn username_register_works<T: GistDatabase>(
    db: &T,
    username: &str,
    password: &str,
    secret: &str,
) {
    let _ = db.delete_account(username).await;
    assert!(matches!(
        db.username_login(username).await.err(),
        Some(DBError::AccountNotFound)
    ));

    let mut register_payload = UsernameRegisterPayload {
        username,
        password,
        secret,
    };

    db.username_register(&register_payload).await.unwrap();
    assert!(db.username_exists(username).await.unwrap());
    assert_eq!(db.get_secret(username).await.unwrap(), secret);
    let login_resp = db.username_login(username).await.unwrap();
    assert_eq!(login_resp.password, password);

    register_payload.secret = username;
    assert!(matches!(
        db.username_register(&register_payload).await.err(),
        Some(DBError::DuplicateUsername)
    ));
}

/// test duplicate secret errors
pub async fn duplicate_secret_guard_works<T: GistDatabase>(
    db: &T,
    username: &str,
    password: &str,
    username2: &str,
    secret: &str,
    duplicate_secret: &str,
) {
    let _ = db.delete_account(username).await;
    let _ = db.delete_account(username2).await;

    let mut register_payload = UsernameRegisterPayload {
        username,
        password,
        secret,
    };

    db.username_register(&register_payload).await.unwrap();
    assert!(db.username_exists(username).await.unwrap());
    assert_eq!(db.get_secret(username).await.unwrap(), secret);

    register_payload.username = username2;
    assert!(matches!(
        db.username_register(&register_payload).await.err(),
        Some(DBError::DuplicateSecret)
    ));

    assert!(matches!(
        db.update_secret(username, duplicate_secret).await.err(),
        Some(DBError::DuplicateSecret)
    ));

    db.update_secret(username, username).await.unwrap();
}

/// check if duplicate username and duplicate email guards are working on update workflows

pub async fn duplicate_username_and_email<T: GistDatabase>(
    db: &T,
    username: &str,
    fresh_username: &str,
    fresh_email: &str,
    password: &str,
    secret: &str,
    duplicate_username: &str,
    duplicate_email: &str,
) {
    let _ = db.delete_account(username).await;
    let _ = db.delete_account(fresh_username).await;
    let register_payload = UsernameRegisterPayload {
        username,
        password,
        secret,
    };

    db.username_register(&register_payload).await.unwrap();

    let mut update_email_payload = UpdateEmailPayload {
        username,
        email: duplicate_email,
    };
    let err = db.update_email(&update_email_payload).await.err();
    assert!(matches!(err, Some(DBError::DuplicateEmail)));
    update_email_payload.email = fresh_email;
    db.update_email(&update_email_payload).await.unwrap();

    let mut update_username_payload = UpdateUsernamePayload {
        new_username: duplicate_username,
        old_username: username,
    };
    assert!(matches!(
        db.update_username(&update_username_payload).await.err(),
        Some(DBError::DuplicateUsername)
    ));
    update_username_payload.new_username = fresh_username;
    db.update_username(&update_username_payload).await.unwrap();
}
