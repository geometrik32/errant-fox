// @generated — matches migrations/0001_initial + 0002_comment_reactions

diesel::table! {
    users (id) {
        id            -> Text,
        username      -> Text,
        display_name  -> Text,
        password_hash -> Text,
        is_admin      -> Bool,
        avatar_path   -> Nullable<Text>,
        color         -> Nullable<Text>,
        created_at    -> Timestamp,
    }
}

diesel::table! {
    videos (id) {
        id            -> Text,
        seafile_path  -> Text,
        fighter_a_id  -> Nullable<Text>,
        fighter_b_id  -> Nullable<Text>,
        date          -> Date,
        duration_ms   -> Nullable<Integer>,
        preview_count -> Integer,
        created_at    -> Timestamp,
    }
}

diesel::table! {
    techniques (id) {
        id   -> Integer,
        name -> Text,
    }
}

diesel::table! {
    bouts (id) {
        id             -> Integer,
        video_id       -> Text,
        order_index    -> Integer,
        time_start_ms  -> Integer,
        time_end_ms    -> Integer,
        score_a        -> Integer,
        score_b        -> Integer,
        technique_a_id -> Nullable<Integer>,
        technique_b_id -> Nullable<Integer>,
        hit_zone_a     -> Nullable<Text>,
        hit_zone_b     -> Nullable<Text>,
        result_a       -> Nullable<Text>,
        result_b       -> Nullable<Text>,
    }
}

diesel::table! {
    comments (id) {
        id           -> Integer,
        video_id     -> Text,
        author_id    -> Text,
        timestamp_ms -> Integer,
        text         -> Text,
        reply_to_id  -> Nullable<Integer>,
        created_at   -> Timestamp,
        edited_at    -> Nullable<Timestamp>,
    }
}

diesel::table! {
    comment_reactions (comment_id, user_id) {
        comment_id -> Integer,
        user_id    -> Text,
        kind       -> Text,
    }
}

// Unambiguous FK joins (multi-FK paths use manual aliases in queries)
diesel::joinable!(bouts             -> videos   (video_id));
diesel::joinable!(comments          -> videos   (video_id));
diesel::joinable!(comments          -> users    (author_id));
diesel::joinable!(comment_reactions -> comments (comment_id));
diesel::joinable!(comment_reactions -> users    (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    videos,
    techniques,
    bouts,
    comments,
    comment_reactions,
);
