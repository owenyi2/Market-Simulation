import threading 
import requests
import time
import numpy as np

MAX_ORDERS = 10
NUM_AGENTS = 70

def zero_intelligence_agent(account_init):
    FRACTION = 0.2
    SPREAD = 0.05

    response = requests.post('http://127.0.0.1:3000/api/account/new', headers={'Content-Type': 'application/json'}, json=account_init) 
    account_id = response.content.decode("utf-8")

    limit = np.random.normal(100, 10)
    first_order = requests.post("http://127.0.0.1:3000/api/order/new", headers={'Content-Type': 'application/json', 'account-id': account_id}, json={"limit": limit, "quantity": 1, "side": "Ask" if np.random.binomial(1, .5) == 1 else "Bid"})
    
    previous_price = limit 

    while True:
        time.sleep(np.random.poisson(0.5))
       
        r = requests.get("http://127.0.0.1:3000/api/order", headers={'account-id': account_id})
        current_orders = r.json()
        # print("[GET] /order", r.elapsed.total_seconds())

        r = requests.get("http://127.0.0.1:3000/api/account", headers={'account-id': account_id})
        account = r.json()
        # print("[GET] /acconut", r.elapsed.total_seconds())

        num_curr_orders = len(current_orders)
        account_balance = account["account_balance"]

        if num_curr_orders + 2 > MAX_ORDERS:
            current_orders.sort(key = lambda x: x["timestamp"], reverse=True)
            oldest_order_id  = current_orders[0]["id"]
            r = requests.delete(f"http://127.0.0.1:3000/api/order/{oldest_order_id}", headers={'account-id': account_id})
            # print("[DELETE] /order/:id", r.elapsed.total_seconds()) 
            continue
        else: 
            r = requests.get("http://127.0.0.1:3000/api/market/quote")
            quotes = r.json()
            # print("[GET] /market/quote", r.elapsed.total_seconds())
            if quotes[0] == None:
                quotes[0] = {"quantity": 1, "limit": previous_price}
            if quotes[1] == None:            
                quotes[1] = {"quantity": 1, "limit": previous_price}

            limit = (quotes[0]["limit"] + quotes[1]["limit"]) / 2 * (1 + np.random.normal(0, 1) * 0.01)
            r = requests.post("http://127.0.0.1:3000/api/order/new", headers={'Content-Type': 'application/json', 'account-id': account_id}, json={"limit": limit, "quantity": 1, "side": "Ask" if np.random.binomial(1, .5) == 1 else "Bid"})
            print("[POST] /order/new", r.elapsed.total_seconds())

account_init = {
    'account_balance': 10000.0,
    'position': 0,
}
# zero_intelligence_agent(account_init)
for i in range(NUM_AGENTS):
    time.sleep(np.random.poisson(1))
    thread = threading.Thread(target=zero_intelligence_agent, args=(account_init, ))
    thread.start()
