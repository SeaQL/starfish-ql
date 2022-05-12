# StarfishQL

## Testing

We have a shell script, `test.sh`, for you to perform testing easily.

```sh
$ test.sh mysql     # Test it on MySQL
$ test.sh postgres  # Test it on PostgreSQL
$ test.sh sqlite    # Test it on SQLite
```

## Running REST API

First, you need to set the connection string for your database in `Rocket.toml`:

```toml
[default.databases.starfish]
url = "mysql://root:root@localhost/starfish"
# url = "postgres://root:root@localhost/starfish"
# url = "sqlite:./starfish.db?mode=rwc"
```

Then, run the application with one or more `DATABASE_DRIVER` from:

- `sqlx-mysql` - SQLx MySQL
- `sqlx-postgres` - SQLx PostgreSQL
- `sqlx-sqlite` - SQLx SQLite

```sh
$ cargo run --features=<DATABASE_DRIVER>
```

If you want to setup your database with Docker:

```sh
$ docker run \
    --name "mysql-8.0" \
    --env MYSQL_DB="mysql" \
    --env MYSQL_USER="sea" \
    --env MYSQL_PASSWORD="sea" \
    --env MYSQL_ALLOW_EMPTY_PASSWORD="yes" \
    --env MYSQL_ROOT_PASSWORD="root" \
    -d -p 3306:3306 mysql:8.0
docker stop "mysql-8.0"
```

```sh
$ docker run \
    --name "mariadb-10.6" \
    --env MYSQL_DB="mysql" \
    --env MYSQL_USER="sea" \
    --env MYSQL_PASSWORD="sea" \
    --env MYSQL_ALLOW_EMPTY_PASSWORD="yes" \
    --env MYSQL_ROOT_PASSWORD="root" \
    -d -p 3306:3306 mariadb:10.6
docker stop "mariadb-10.6"
```

```sh
$ docker run \
    --name "postgres-13" \
    --env POSTGRES_USER="root" \
    --env POSTGRES_PASSWORD="root" \
    -d -p 5432:5432 postgres:13
docker stop "postgres-13"
```
