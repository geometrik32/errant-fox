-- Down migration
DROP TRIGGER IF EXISTS tr_hema_delete_participant_on_user_delete;
DROP TRIGGER IF EXISTS tr_hema_delete_user_on_participant_delete;
