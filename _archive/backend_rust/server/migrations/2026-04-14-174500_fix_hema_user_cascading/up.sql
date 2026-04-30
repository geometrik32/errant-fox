-- Up migration to enforce 1:1 inseparable relationship between users and hema_participants

-- 1. Trigger to delete participant when user is deleted
CREATE TRIGGER IF NOT EXISTS tr_hema_delete_participant_on_user_delete 
AFTER DELETE ON users
FOR EACH ROW
BEGIN
    DELETE FROM hema_participants WHERE account_id = OLD.id;
END;

-- 2. Trigger to delete user when participant is deleted
CREATE TRIGGER IF NOT EXISTS tr_hema_delete_user_on_participant_delete
AFTER DELETE ON hema_participants
FOR EACH ROW
WHEN OLD.account_id IS NOT NULL
BEGIN
    DELETE FROM users WHERE id = OLD.account_id;
END;
