# StarfishQL

## Testing

```sh
DATABASE_URL="mysql://root:root@localhost:3306" cargo t --all
```

## Running REST API

```sh
cargo r
```

You might need to setup your database with Docker:

```sh
docker run \
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
docker run \
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
docker run \
    --name "postgres-13" \
    --env POSTGRES_USER="root" \
    --env POSTGRES_PASSWORD="root" \
    -d -p 5432:5432 postgres:13
docker stop "postgres-13"
```

## REST API docs

https://documenter.getpostman.com/view/15661872/UVRGFjWR
