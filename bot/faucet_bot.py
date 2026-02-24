#!/usr/bin/env python3
"""
MERKLITH Faucet Bot - Automatic Transaction Generator

This bot creates continuous activity on the blockchain:
1. Sends funds from faucet account to random addresses
2. Creates transactions at regular intervals
3. Triggers block production

Usage:
  python3 faucet_bot.py

Settings:
  - INTERVAL_MIN: Minimum wait time (seconds)
  - INTERVAL_MAX: Maximum wait time (seconds)
  - MIN_AMOUNT: Minimum transfer amount (ANV)
  - MAX_AMOUNT: Maximum transfer amount (ANV)
"""

import requests
import json
import random
import time
import sys
from datetime import datetime

# Configuration
RPC_URL = "http://localhost:9999"  # CORS Proxy
FAUCET_ADDRESS = "0x09bcc216d0fbdcbe6fb5d65e993760b30bec7722"
FAUCET_PRIVATE_KEY = (
    "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
)

# Random transfer targets (dummy addresses)
TARGET_ADDRESSES = [
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    "0x8ba1f109551bD432803012645Ac136ddd64DBA72",
    "0xdD870fA1b7C4700F2BD7f44238821C26f7392148",
    "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B",
    "0x1aB489E589De6E2F9c9b6B9e2F2b1a4c3d5E6F78",
    "0x2Bc5901A6E4984628Bf12C539f06D5b3369eD0C1",
    "0x3Cd601A7E5985739Bf13D54A107d5b4479fE1D2E",
    "0x4DE710A8E6A96849Cf15D54B208e6C548aF2E3F4",
    "0x5EF820B9F7BA0706A1c4D8c59e3D4A0c40aF3e6b",
    "0x6aD931F4c8AB1507a3b2C5d6E7F8A9B0C1D2E3F4",
]

# Timing configuration
INTERVAL_MIN = 5  # Minimum 5 seconds
INTERVAL_MAX = 25  # Maximum 25 seconds (average 15s)

# Transfer amounts
MIN_AMOUNT = 0.001  # 0.001 ANV
MAX_AMOUNT = 0.1  # 0.1 ANV


class FaucetBot:
    def __init__(self):
        self.tx_count = 0
        self.total_transferred = 0.0
        self.start_time = time.time()

    def generate_random_address(self):
        """Select random target address"""
        return random.choice(TARGET_ADDRESSES)

    def generate_amount(self):
        """Generate random transfer amount"""
        return random.uniform(MIN_AMOUNT, MAX_AMOUNT)

    def send_transaction(self, to_address, amount):
        """Send transaction via RPC"""
        try:
            # Convert ANV to smallest unit (assuming 18 decimals)
            amount_wei = hex(int(amount * 10**18))

            payload = {
                "jsonrpc": "2.0",
                "method": "merklith_transfer",
                "params": [FAUCET_ADDRESS, to_address, amount_wei],
                "id": 1,
            }

            response = requests.post(
                RPC_URL,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=10,
            )

            if response.status_code == 200:
                result = response.json()
                if "result" in result:
                    return True, result["result"]
                elif "error" in result:
                    return False, result["error"]["message"]
            return False, f"HTTP {response.status_code}"

        except Exception as e:
            return False, str(e)

    def print_status(self):
        """Print current status"""
        elapsed = time.time() - self.start_time
        tx_per_hour = (self.tx_count / elapsed) * 3600 if elapsed > 0 else 0

        stats = {
            "tx_count": self.tx_count,
            "total_transferred": self.total_transferred,
            "elapsed_hours": elapsed / 3600,
            "tx_per_hour": tx_per_hour,
        }

        print("\n" + "=" * 60)
        print("FAUCET BOT STATUS")
        print("=" * 60)
        print(f"Transactions: {stats['tx_count']}")
        print(f"Total Transferred: {stats['total_transferred']:.4f} ANV")
        print(f"Running: {stats['elapsed_hours']:.2f} hours")
        print(f"TX/Hour: {stats['tx_per_hour']:.2f}")
        print(f"{'=' * 60}\n")

    def run(self):
        """Main loop"""
        print("MERKLITH Faucet Bot Started!")
        print(f"Faucet: {FAUCET_ADDRESS}")
        print(f"Targets: {len(TARGET_ADDRESSES)} addresses")
        print(f"Interval: {INTERVAL_MIN}-{INTERVAL_MAX} seconds")
        print(f"Amount: {MIN_AMOUNT}-{MAX_AMOUNT} ANV")
        print(f"RPC: {RPC_URL}")
        print("\nStarting transfers... (Ctrl+C to stop)\n")

        try:
            while True:
                # Random target and amount
                target = self.generate_random_address()
                amount = self.generate_amount()

                # Transfer
                print(
                    f"[{datetime.now().strftime('%H:%M:%S')}] Sending {amount:.4f} ANV to {target[:20]}...",
                    end=" ",
                )

                success, result = self.send_transaction(target, amount)

                if success:
                    self.tx_count += 1
                    self.total_transferred += amount
                    print(f"OK TX: {result[:20]}...")
                else:
                    print(f"ERROR: {result}")

                # Show stats every 10 transactions
                if self.tx_count % 10 == 0:
                    self.print_status()

                # Random wait time
                wait_time = random.uniform(INTERVAL_MIN, INTERVAL_MAX)
                time.sleep(wait_time)

        except KeyboardInterrupt:
            print("\n\nBot stopped by user")
            self.print_status()
            sys.exit(0)
        except Exception as e:
            print(f"\nFatal error: {e}")
            sys.exit(1)


def main():
    bot = FaucetBot()
    bot.run()


if __name__ == "__main__":
    main()
