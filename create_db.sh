#!/bin/bash


function show_help {
    echo "Usage:"
    echo "   create_db.sh [-h, --help] [db-name] [db-user] [db-user-pass]"
    echo ""
    echo "Ex."
    echo "   create_db.sh -h                      -> show this help"
    echo "   create_db.sh --default               -> create db/user with default values (upaste/upaste/upaste-pass)"
    echo "   create_db.sh db_name db_user db_pass -> create db/user with given params"
}


if [[ -z $1 ]]
then
    show_help
    exit 0
else
    if [[ "$1" == "--help" || "$1" == "-h" ]]
    then
        show_help
        exit 0
    elif [[ "$1" == "--default" ]]
    then
        db_name="upaste"
        db_user="upaste"
        db_pass="upaste-pass"
    else
        db_name=$1
        if [[ $2 ]]
        then
            db_user=$2
        fi
        if [[ $3 ]]
        then
            db_pass=$3
        fi
    fi
fi


echo "* Creating db: $db_name" &&
sudo -u postgres createdb $db_name &&

echo "* Creating db-user: $db_user" &&
sudo -u postgres createuser $db_user &&

echo "* Setting db-user password: $db_pass" &&
sudo -u postgres psql -c "alter user $db_user with password '$db_pass';"

