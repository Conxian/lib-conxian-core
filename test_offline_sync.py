import requests
import time
import subprocess
import os
import uuid
import json
from datetime import datetime, timedelta, timezone

def test_offline_sync():
    print("🚀 Starting 48-hour Blackout Forensic Test (CON-75)...")
    
    # 1. Start the gateway in "testing" and "offline" mode
    env = os.environ.copy()
    env["CONXIAN_TESTING"] = "true"
    env["CONXIAN_OFFLINE"] = "true"
    # Make sure cargo is in PATH
    cargo_path = "/Users/composter/.rustup/toolchains/stable-x86_64-apple-darwin/bin"
    env["PATH"] = f"{cargo_path}:{env.get('PATH', '')}"

    print("[-] Starting Gateway in Offline mode...")
    process = subprocess.Popen(["cargo", "run"], cwd="gateway", env=env)
    
    # Wait for gateway to be ready
    print("[-] Waiting for gateway to compile and start...")
    ready = False
    for i in range(120): # Up to 2 minutes for compilation
        try:
            res = requests.get("http://localhost:8080/api/v1/health", timeout=1)
            if res.status_code == 200:
                ready = True
                break
        except:
            pass
        time.sleep(2)
    
    if not ready:
        print("❌ Gateway failed to start in time.")
        process.terminate()
        return False
    
    print("✅ Gateway is READY.")

    try:
        # 2. Inject 500 transactions spread over 48 hours
        print("[-] Injecting 500 transactions spread over 48 hours...")
        start_time = datetime(2026, 3, 28, 12, 0, 0, tzinfo=timezone.utc)
        
        for i in range(500):
            # Spread 500 transactions over 48 hours (approx 1 every 5-6 mins)
            current_time = start_time + timedelta(seconds=i * (48 * 3600 // 500))
            job_card = {
                "id": str(uuid.uuid4()),
                "context": "https://conxian.com/contexts/job-card/v2.0",
                "type_name": "ConxianJobCard",
                "version": "2.0.0",
                "status": "Pending",
                "tx_hash": f"{i:08x}_test_blackout",
                "amount_stx": 10.5 + (i % 5),
                "timestamp": current_time.isoformat().replace("+00:00", "Z"),
                "signature": f"sig_mock_biometric_passkey_2026_{i:04x}"
            }
            res = requests.post("http://localhost:8080/api/v1/pos/sync", json=job_card)
            if res.status_code != 200:
                print(f"❌ Error injecting transaction {i}: {res.text}")
                return False
            if i % 100 == 0:
                print(f"    - {i} transactions injected...")

        # 3. Check mesh status
        print("[-] Checking Mesh Status (Gossip check)...")
        res = requests.get("http://localhost:8080/api/v1/mesh/status")
        mesh_status = res.json()
        print(f"    - Mesh status: {mesh_status['status']}")
        print(f"    - Synced hashes in mesh: {len(mesh_status['synced_hashes'])}")
        
        # 4. "Go online"
        print("[-] Simulating Backhaul Restoration (Switching to Online)...")
        process.terminate()
        process.wait()
        
        print("[-] Restarting Gateway in ONLINE mode...")
        env.pop("CONXIAN_OFFLINE", None)
        process = subprocess.Popen(["cargo", "run"], cwd="gateway", env=env)
        
        # Wait for gateway to be ready
        ready = False
        for i in range(120):
            try:
                res = requests.get("http://localhost:8080/api/v1/health", timeout=1)
                if res.status_code == 200:
                    ready = True
                    break
            except:
                pass
            time.sleep(2)
        
        if not ready:
            print("❌ Gateway failed to restart in time.")
            return False
        
        print("✅ Gateway is ONLINE.")
        time.sleep(10) # Wait for sync cycles

        # 5. Verify sync
        print("[-] Verifying all 500 transactions are synced to L2...")
        
        # Wait for sync cycles (up to 60 seconds)
        max_retries = 30
        synced_count = 0
        for i in range(max_retries):
            res = requests.get("http://localhost:8080/api/v1/pos/sync/status")
            jobs = res.json()
            synced_count = sum(1 for job in jobs if job["status"] == "Synced")
            print(f"    - Sync progress: {synced_count}/500 jobs synced...")
            if synced_count >= 500:
                break
            time.sleep(2)
        
        if synced_count >= 500:
            print("✅ All 500 transactions successfully synced after blackout!")
            print("✅ Forensic test completed successfully.")
            return True
        else:
            print(f"❌ Sync failed. Only {synced_count}/500 jobs synced.")
            return False

    finally:
        process.terminate()

if __name__ == "__main__":
    test_offline_sync()
