 CREATE TABLE IF NOT EXISTS questions (
    id serial PRIMARY KEY,               
    title VARCHAR (255) NOT NULL,
    content TEXT NOT NULL,
    tags TEXT [],
    created_on TIMESTAMP NOT NULL DEFAULT NOW()   
);

CREATE TABLE IF NOT EXISTS answers (
    id serial PRIMARY KEY,
    content TEXT NOT NULL,
    created_on TIMESTAMP NOT NULL DEFAULT NOW(),
    corresponding_question integer REFERENCES questions
 );

 CREATE TABLE IF NOT EXISTS accounts (
    id serial NOT NULL,
    email VARCHAR(255) NOT NULL PRIMARY KEY,
    password VARCHAR(255) NOT NULL
 );

 CREATE TABLE IF NOT EXISTS passwords (
  client_id TEXT PRIMARY KEY,
  client_secret TEXT NOT NULL,
  full_name TEXT NOT NULL,
  email TEXT NOT NULL
);