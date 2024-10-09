FROM postgres:16.1-alpine3.19

RUN mkdir -p /docker-entrypoint-initdb.d
COPY initialize_database.sh /docker-entrypoint-initdb.d/
RUN chown -R postgres:postgres /docker-entrypoint-initdb.d

EXPOSE 5432
