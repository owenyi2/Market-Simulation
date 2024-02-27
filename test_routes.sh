#!/bin/bash
# echo "[POST] /account/new"
# curl \
#     -H 'Content-Type: application/json' \
#     -d '{ "account_balance": 1000.0, "position": 0 }' \
#     -X POST \
#     "http://localhost:3000/api/account/new"

echo "[GET] /account/"
curl \
    -H 'aiccount-id: urmom' \
    -X GET \
    "http://localhost:3000/api/account"
