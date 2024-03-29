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

echo "[POST] /api/order/new"
ORDER_1_0=$(curl -s \
    -H 'Content-Type: application/json' \
    -H "account-id: ${ACCOUNT_ID_1}" \
    -d '{ "limit": 12.01, "quantity": 10, "side": "Bid" }' \
    -X POST \
    "http://localhost:3000/api/order/new")

echo $ORDER_1_0

echo "[POST] /api/order/new"
# This request intentionally fails
curl \
    -H 'Content-Type: application/json' \
    -H "account-id: ${ACCOUNT_ID_1}" \
    -d '{ "limit": 11.99, "quantity": 10, "side": "Ask" }' \
    -X POST \
    "http://localhost:3000/api/order/new" && echo


ORDER_1_0=$(echo $ORDER_1_0 | grep -Eo '"id":".*?"' | grep -o '[a-z0-9\-]*' | tail -n1) 

echo "[POST] /api/order/new"
ORDER_2_0=$(curl -s \
    -H 'Content-Type: application/json' \
    -H "account-id: ${ACCOUNT_ID_2}" \
    -d '{ "limit": 12.00, "quantity": 8, "side": "Ask" }' \
    -X POST \
    "http://localhost:3000/api/order/new")

echo $ORDER_2_0

ORDER_2_0=$(echo $ORDER_2_0 | grep -Eo '"id":".*?"' | grep -o '[a-z0-9\-]*' | tail -n1) 
echo $ORDER_2_0
echo "[GET] /account/"

curl \
    -H "account-id: ${ACCOUNT_ID_1}" \
    -X GET \
    "http://localhost:3000/api/account" && echo

echo "[GET] /account/"
curl -s \
    -H "account-id: ${ACCOUNT_ID_2}" \
    -X GET \
    "http://localhost:3000/api/account" && echo

echo "[GET] /order/"
curl \
    -H "account-id: ${ACCOUNT_ID_1}" \
    -X GET \
    "http://localhost:3000/api/order" && echo

echo "[GET] /order/:id"
curl \
    -H "account-id: ${ACCOUNT_ID_2}" \
    -X GET \
    "http://localhost:3000/api/order/${ORDER_2_0}" && echo

echo "[DELETE] /order/:id"
curl \
    -H "account-id: ${ACCOUNT_ID_1}" \
    -X DELETE \
    "http://localhost:3000/api/order/${ORDER_1_0}"

echo "[DELETE] /order/:id"
# This request intentionally fails
curl \
    -H "account-id: ${ACCOUNT_ID_2}" \
    -X DELETE \
    "http://localhost:3000/api/order/${ORDER_2_0}" && echo

echo "[POST] /api/order/new"
ORDER_1_1=$(curl -s \
    -H 'Content-Type: application/json' \
    -H "account-id: ${ACCOUNT_ID_1}" \
    -d '{ "limit": 11.81, "quantity": 10, "side": "Bid" }' \
    -X POST \
    "http://localhost:3000/api/order/new")

echo $ORDER_1_1

echo "[POST] /api/order/new"
ORDER_1_2=$(curl -s \
    -H 'Content-Type: application/json' \
    -H "account-id: ${ACCOUNT_ID_1}" \
    -d '{ "limit": 12.31, "quantity": 10, "side": "Ask" }' \
    -X POST \
    "http://localhost:3000/api/order/new")

echo $ORDER_1_2

echo "[GET] /market/quote"
curl \
    -X GET \
    "http://localhost:3000/api/market/quote"
