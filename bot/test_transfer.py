#!/usr/bin/env python3
"""Test RPC transfer methods"""

import requests
import json

RPC_URL = "http://localhost:8545"


def test_transfer():
    # Test data
    from_addr = "0xdD870fA1b7C4700F2BD7f44238821C26f7392148"
    to_addr = "0x09bcc216d0fbdcbe6fb5d65e993760b30bec7722"
    amount = "0x56BC75E2D63100000"  # 100 ANV

    # Test 1: merklith_transfer with array params
    print("Test 1: merklith_transfer")
    payload = {
        "jsonrpc": "2.0",
        "method": "merklith_transfer",
        "params": [from_addr, to_addr, amount],
        "id": 1,
    }
    resp = requests.post(
        RPC_URL, json=payload, headers={"Content-Type": "application/json"}
    )
    print(f"  Status: {resp.status_code}")
    print(f"  Response: {resp.json()}")

    # Test 2: eth_sendTransaction with object param
    print("\nTest 2: eth_sendTransaction")
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_sendTransaction",
        "params": [{"from": from_addr, "to": to_addr, "value": amount}],
        "id": 1,
    }
    resp = requests.post(
        RPC_URL, json=payload, headers={"Content-Type": "application/json"}
    )
    print(f"  Status: {resp.status_code}")
    print(f"  Response: {resp.json()}")

    # Test 3: Check if from address exists
    print("\nTest 3: Check from address balance")
    payload = {
        "jsonrpc": "2.0",
        "method": "merklith_getBalance",
        "params": [from_addr],
        "id": 1,
    }
    resp = requests.post(
        RPC_URL, json=payload, headers={"Content-Type": "application/json"}
    )
    result = resp.json()
    if "result" in result:
        balance = int(result["result"], 16) / 10**18
        print(f"  Balance: {balance} ANV")
    else:
        print(f"  Error: {result}")


if __name__ == "__main__":
    test_transfer()
