import psycopg2
import json

with open("config.json") as f:
    conn_string = json.loads(f.read())["db"]["connection_string"]

username = input("Username: ")
permissions = input("Permissions (comma separated): ").split(",")

conn = psycopg2.connect(conn_string)

cur = conn.cursor()

cur.execute(f"""UPDATE users SET permissions = '{{{",".join(permissions)}}}' WHERE username = '{username}'""")

cur.execute(f"""SELECT * FROM users WHERE username = '{username}'""")

rows = cur.fetchall()
conn.commit()
cur.close()
conn.close()
print(rows)