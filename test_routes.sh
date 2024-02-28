#!/bin/bash
echo "[POST] /account/new"
ACCOUNT_ID_1=$(curl -s \
    -H 'Content-Type: application/json' \
    -d '{ "account_balance": 1000.0, "position": 0 }' \
    -X POST \
    "http://localhost:3000/api/account/new")
echo $ACCOUNT_ID_1

echo "[POST] /account/new"
ACCOUNT_ID_2=$(curl -s \
    -H 'Content-Type: application/json' \
    -d '{ "account_balance": 2000.0, "position": 0 }' \
    -X POST \
    "http://localhost:3000/api/account/new")
echo $ACCOUNT_ID_2

echo "[GET] /account/"
curl \
    -H "account-id: ${ACCOUNT_ID_1}" \
    -X GET \
    "http://localhost:3000/api/account" && echo

echo "[GET] /account/"
curl \
    -H "account-id: ${ACCOUNT_ID_2}" \
    -X GET \
    "http://localhost:3000/api/account" && echo
