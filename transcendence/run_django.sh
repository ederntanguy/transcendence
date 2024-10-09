#!/bin/bash
set -e

echo "postgres:5432:transcendence:transcendence:${POSTGRES_TRANSCENDENCE_PASSWORD}" > .pgpass
chmod 0600 .pgpass

mkdir -p /home/shared/media/account

python3 manage.py makemigrations
python3 manage.py makemigrations account
python3 manage.py migrate
python3 manage.py collectstatic

psql -U 'transcendence' -d 'transcendence' -W -h postgres -p 5432 -c "INSERT INTO account_player(username, password, tournament_username) VALUES ('deleted', '\x00', 'deleted');" <<-EOSQL
${POSTGRES_TRANSCENDENCE_PASSWORD}
EOSQL

daphne -e ssl:43443:privateKey=ssl/transcendence.key:certKey=ssl/transcendence.crt transcendence.asgi:application
