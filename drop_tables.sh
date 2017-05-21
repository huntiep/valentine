#!/bin/bash

psql -d valentine -c "DROP TABLE users"
psql -d valentine -c "DROP TABLE repos"
