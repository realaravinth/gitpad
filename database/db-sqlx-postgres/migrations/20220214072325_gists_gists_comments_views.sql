CREATE OR REPLACE VIEW gists_gists_view AS
    SELECT
        gists.description,
        gists.created,
        gists.updated,
        gists.public_id,
        gists_users.username as owner,
        gists_privacy.name as privacy
    FROM gists_gists gists
    INNER JOIN gists_privacy ON gists_privacy.ID = gists.privacy
    INNER JOIN gists_users ON gists_users.ID = gists.owner_id;


CREATE OR REPLACE VIEW gists_comments_view AS
    SELECT
        gists_comments.ID,
        gists_comments.comment,
        gists_comments.created,
        gists_gists.public_id as gist_public_id,
        gists_gists.ID as gist_id,
        gists_users.username as owner
    FROM gists_comments gists_comments
        INNER JOIN gists_users ON gists_users.ID = gists_comments.owner_id
        INNER JOIN gists_gists ON gists_gists.ID = gists_comments.gist_id;
