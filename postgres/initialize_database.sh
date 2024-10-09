#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 -U "${POSTGRES_USER}" --dbname "$POSTGRES_DB" <<-EOSQL
	CREATE USER root;
	CREATE DATABASE root;
	CREATE USER transcendence WITH PASSWORD '${POSTGRES_TRANSCENDENCE_PASSWORD}';
	GRANT ALL PRIVILEGES ON DATABASE transcendence TO transcendence;
	ALTER DATABASE transcendence OWNER TO transcendence;
EOSQL
