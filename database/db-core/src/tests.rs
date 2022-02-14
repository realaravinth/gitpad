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

/// test if all visibility modes are available on database
pub async fn visibility_works<T: GistDatabase>(db: &T) {
    for p in [
        GistVisibility::Public,
        GistVisibility::Unlisted,
        GistVisibility::Private,
    ]
    .iter()
    {
        println!("Testing visibility: {}", p.to_str());
        assert!(db.visibility_exists(p).await.unwrap());
    }
}

/// test all gist methods
pub async fn gists_work<T: GistDatabase>(
    db: &T,
    username: &str,
    password: &str,
    secret: &str,
    public_id: &str,
) {
    fn assert_comments(lhs: &CreateGistComment, rhs: &GistComment) {
        println!("lhs: {:?} rhs: {:?}", lhs, rhs);
        assert_eq!(rhs.owner, lhs.owner);
        assert_eq!(rhs.comment, lhs.comment);
        assert_eq!(rhs.gist_public_id, lhs.gist_public_id);
    }

    fn assert_gists(lhs: &CreateGist, rhs: &Gist) {
        assert_eq!(lhs.description.as_ref().unwrap(), rhs.description.as_ref().unwrap());
        assert_eq!(lhs.owner, rhs.owner);
        assert_eq!(lhs.public_id, rhs.public_id);
        assert_eq!(lhs.visibility, &rhs.visibility);
    }

    let _ = db.delete_account(username).await;
    let register_payload = UsernameRegisterPayload {
        username,
        password,
        secret,
    };

    db.username_register(&register_payload).await.unwrap();

    let create_gist = CreateGist {
        owner: username.into(),
        description: Some("foo"),
        public_id,
        visibility: &GistVisibility::Public,
    };

    assert!(!db.gist_exists(&create_gist.public_id).await.unwrap());
    // create gist
    assert!(db.get_user_gists(username).await.unwrap().is_empty());

    db.new_gist(&create_gist).await.unwrap();
    assert!(matches!(
        db.new_gist(&create_gist).await.err(),
        Some(DBError::GistIDTaken)
    ));

    assert!(db.gist_exists(&create_gist.public_id).await.unwrap());
    // get gist
    let db_gist = db.get_gist(&create_gist.public_id).await.unwrap();
    assert_gists(&create_gist, &db_gist);

    let mut gists = db.get_user_gists(username).await.unwrap();
    assert_eq!(gists.len(), 1);
    let gist = gists.pop().unwrap();
    assert_gists(&create_gist, &gist);

    // comment on gist
    let create_comment = CreateGistComment {
        owner: username.into(),
        gist_public_id: create_gist.public_id.clone(),
        comment: "foo".into(),
    };
    db.new_comment(&create_comment).await.unwrap();
    // get all comments on gist
    let mut comments = db
        .get_comments_on_gist(&create_gist.public_id)
        .await
        .unwrap();
    assert!(comments.len() == 1);
    let comment = comments.pop().unwrap();
    assert_comments(&create_comment, &comment);

    // get all comments by ID
    let comment = db.get_comment_by_id(comment.id).await.unwrap();
    assert_comments(&create_comment, &comment);

    // delete comment
    db.delete_comment(username, comment.id).await.unwrap();

    assert!(matches!(
        db.get_comment_by_id(comment.id).await.err().unwrap(),
        DBError::CommentNotFound
    ));

    //  delete gist
    db.delete_gist(username, &create_gist.public_id)
        .await
        .unwrap();
    assert!(matches!(
        db.get_gist(&create_gist.public_id).await.err().unwrap(),
        DBError::GistNotFound
    ));
    assert!(db
        .get_comments_on_gist(&create_gist.public_id)
        .await
        .unwrap()
        .is_empty());
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
