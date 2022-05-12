#!/bin/bash

if [[ $1 == mysql ]]; then
    DATABASE_URL="mysql://root:root@localhost:3306" cargo t --all --features=sqlx-mysql
elif [[ $1 == postgres ]]; then
    DATABASE_URL="postgres://root:root@localhost:5432" cargo t --all --features=sqlx-postgres
elif [[ $1 == sqlite ]]; then
    DATABASE_URL="sqlite:./sqlite/" cargo t --all --features=sqlx-sqlite 
else
    echo "Unknown DB Backend."
fi
