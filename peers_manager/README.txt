Navigate DB:
psql -h localhost -U postgres -d postgres

Run DB:
docker run --rm   --name pg-docker -e POSTGRES_PASSWORD=docker -d -p 5432:5432 -v $HOME/docker/volumes/postgres:/var/lib/postgresql/data  postgres

Create user:
curl -X POST \
http://127.0.0.1:8080/users/create_user \
-H 'Content-Type: application/json' \
-H 'auth-token: <your auth token>' \
-d '{"name": "name1", "email": "email1", "about": "about1"}'

Login user:
curl -X POST \
http://127.0.0.1:8080/users/login \
-H 'Content-Type: application/json' \
-H 'auth-token: <your auth token>' \
-d '{"email": "email1"}'

User search:
curl -X POST \
http://127.0.0.1:8080/users/search \
-H 'Content-Type: application/json' \
-H 'auth-token: <your auth token>' \
-d '{"keywork": "email"}'
