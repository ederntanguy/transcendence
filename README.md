# Project Transcendence

### Subject version

The subject version can be found on the first page of [the subject](./en.subject.pdf).

### Modules chosen

Minor modules :
- Use the PostgreSQL database engine instead of SQLite
- Use the Bootstrap toolkit
- Supporting another browser
- GDPR compliance

Major modules :
- Use Django as a back-end framework
- User registration and management
- Support users on different computers
- Implement an algorithm to play against
- Implement a pong server and formalize its api

### Run in production

Put the appropriate .env file alongside the docker-compose.yaml file. Put a SSL cerfiticate and its
associated private key called ```transcendence.crt/key``` in the nginx folder and in an `ssl` folder
in the inner transcendence folder.

Then, just hit ```docker compose up --wait``` and it's running! The website can be accessed with
```https://localhost:8080``` only. Note that the database is persisted in a volume. To reset it,
execute ```docker compose down --volumes```, which deletes the volumes of the compose app getting
downed.

### Development environment

You may want to work on the server-side pong implementation or on the Django... thing.

Getting the database running :
- Properly set up a .env file.
- Navigate to postgres/ and build the PostgreSQL image with ```docker build -t db .```.
- Put a copy of .pg_service.conf in your home directory ~, and change the value of key `host` from
`postgres` to `localhost`.
- Make a .pgpass file containing ```localhost:5432:transcendence:transcendence:``` in Django's
folder, alongside its Dockerfile. Complete the line with the password of the PostgreSQL user called
transcendence you put in your .env. Change the permissions of the file to user read/write only with
the command ```chmod 0600 .pgpass```.
- Run this command in a terminal ```docker run --rm -p 6379:6379 redis:7``` to handle the websocket
communication. Set up redirecting the hostname `redis` to `127.0.0.1` by adding it in `/etc/hosts`.
- Change `wss` to `ws` in `static/account/friends_communication.js` around line 42. You may want to
add that file to your `.gitignore` - it really depends on what you think you might forget.

Django :
- Run the database with ```docker run --env-file=../.env --env POSTGRES_DB=transcendence --env
POSTGRES_USER=postgres -p 5432:5432 --rm -v db-data:/var/lib/postgresql/data -v
sck:/var/run/postgresql -it db``` from the `postgres` folder.
- Build the pong server image with ```docker build -t ps .```from the `pong-serv` folder. Run it
with ```docker run -p 8081:8081 --rm -v ./log/:/var/log/pong/ -v sck:/var/run/postgresql -it ps```.
A log folder gets created, so you probably want to run from the folder as well.
- It's up to you on whether you use a venv or not to run Django, however in both cases you need the
`pq` runtime on your machine for `psycopg` to run. Installing it system-wide is the optimal option.
Look for the `libpq5` package on Ubuntu, or search for the equivalent one on your distribution.
- Set the DJANGO_DEBUG environment variable. It can be an empty definition.
- Run the server with ```python manage.py runserver 8080``` for the development server - still ran
by Daphne - from the inner transcendence folder.

Pong server :
- Run the database with ```docker run --env-file=../.env --env POSTGRES_DB=transcendence --env
POSTGRES_USER=postgres -p 5432:5432 --rm -v db-data:/var/lib/postgresql/data -v
./:/var/run/postgresql -it db``` from the `postgres` folder. This puts the database socket in this
folder. Pass this directory to the program.
- Run the Django server by running ```python manage.py runserver 8080``` from the inner
transcendence folder.
- Get to the pong-serv folder... Happy Rust coding!

### The .env file

The necessary variables to define are :
- `DJANGO_SECRET_KEY`
- `POSTGRES_PASSWORD`
- `POSTGRES_TRANSCENDENCE_PASSWORD`
