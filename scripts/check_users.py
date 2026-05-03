import sqlite3

db_path = 'e:/00_Curent project/Errant Fox/backend/errant_fox.sqlite'
try:
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    cursor.execute("SELECT id, username, is_admin FROM users")
    users = cursor.fetchall()
    print("Users found:", users)
    conn.close()
except Exception as e:
    print("Error:", e)
