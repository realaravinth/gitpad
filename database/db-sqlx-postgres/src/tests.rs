use sqlx::postgres::PgPoolOptions;
use std::env;

use crate::*;

use db_core::tests::*;

#[actix_rt::test]
async fn everyting_works() {
    const EMAIL: &str = "postgresuser@foo.com";
    const EMAIL2: &str = "postgresuse2r@foo.com";
    const NAME: &str = "postgresuser";
    const NAME2: &str = "postgresuser2";
    const NAME3: &str = "postgresuser3";
    const NAME4: &str = "postgresuser4";
    const NAME5: &str = "postgresuser5";
    const NAME6: &str = "postgresuser6";
    const NAME7: &str = "postgresuser7";
    const PASSWORD: &str = "pasdfasdfasdfadf";
    const SECRET1: &str = "postgressecret1";
    const SECRET2: &str = "postgressecret2";
    const SECRET3: &str = "postgressecret3";
    const SECRET4: &str = "postgressecret4";

    let url = env::var("POSTGRES_DATABASE_URL").unwrap();
    let pool_options = PgPoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    db.migrate().await.unwrap();
    email_register_works(&db, EMAIL, NAME, PASSWORD, SECRET1, NAME5).await;
    username_register_works(&db, NAME2, PASSWORD, SECRET2).await;
    duplicate_secret_guard_works(&db, NAME3, PASSWORD, NAME4, SECRET3, SECRET2).await;
    duplicate_username_and_email(&db, NAME6, NAME7, EMAIL2, PASSWORD, SECRET4, NAME, EMAIL).await;
    let creds = Creds {
        username: NAME.into(),
        password: SECRET4.into(),
    };
    db.update_password(&creds).await.unwrap();
}

#[actix_rt::test]
async fn visibility_test() {
    let url = env::var("POSTGRES_DATABASE_URL").unwrap();
    let pool_options = PgPoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    db.migrate().await.unwrap();
    visibility_works(&db).await;
}

#[actix_rt::test]
async fn gist_test() {
    const NAME: &str = "postgisttest";
    const PASSWORD: &str = "pasdfasdfasdfadf";
    const SECRET: &str = "postgisttestsecret";
    const PUBLIC_ID: &str = "postgisttestsecret";

    let url = env::var("POSTGRES_DATABASE_URL").unwrap();
    let pool_options = PgPoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    db.migrate().await.unwrap();
    gists_work(&db, NAME, PASSWORD, SECRET, PUBLIC_ID).await;
}
