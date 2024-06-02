# Rust Web Example

Backend port: 8000
Frontend: 8080

To start backend cd into it and then run so long as there is a needed psql db server running. Frontend use trunk serve

## Rust web development repo - Nathan Moes

This repo will serve for the purposes of the class CS510-Rust web development Spring 2024.
This will include following along with the course textbook as well as any assigned materials etc.
The purpose of the repo is to implement the questions (and answers) database + API/service from the Rust web
Development book by Gruber. Persistent database is based on a pqsl database instance. Development for it
was based on running by the offical docker image for psql.

### Environment variables related to PSQL instance for running server

PG_PORT (default = 6565),
PG_PASSWORD,
PG_USER,
PG_HOST

### Environment variables related to API's used

API_LAYER_KEY: used for the bad words api

## Currently developed functions

## Credit to Github Co-Pilot for the creation of questions in questions.json

### RESTful API supporting CRUD

#### Update question

#### Delete question

#### Post question

#### Get question(s)
