# Overview
This service is tight coupled (bad) to the Hardcore Spigot Plugin.
It connects to the database and creates an api endpoint at http://127.0.0.1/timelines that produces stats about each player along with a timeline of events.

The db connection is an env variable that looks like
```sh
DB_URL=postgres://username:password@localhost:5432/hardcore
```

*Note: This is my first rust project... expect it to be bad.*

# How to build Docker
tagname="$(date '+%Y%m%d-%H%M%Z')"
servicename=somc-hardcore-api
docker build --no-cache -t dmgarvis/$servicename:$tagname .
docker push dmgarvis/$servicename:$tagname
docker tag dmgarvis/$servicename:$tagname dmgarvis/$servicename:latest
docker push dmgarvis/$servicename:latest