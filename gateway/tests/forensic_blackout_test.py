import requests
import time
import subprocess
import os
import uuid
import json

def test_offline_sync():
    print("🚀 Starting 48-hour Blackout Forensic Test (CON-75)...")
    
    # Use current date from context (March 30, 2026)
    # Simulate transactions from 48 hours ago (March 28, 2026)
    simulation_start_time = "2026-03-28T12:00:00Z"
    
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
        # 2. Inject 500 transactions from 48h ago
        print(f"[-] Injecting 500 transactions (timestamp: {simulation_start_time})...")
        for i in range(500):
            job_card = {
                "id": str(uuid.uuid4()),
                "context": "https://conxian.com/contexts/job-card/v2.0",
                "type_name": "ConxianJobCard",
                "version": "2.0.0",
                "status": "Pending",
                "tx_hash": f"hash_{i:04x}_test_blackout",
                "amount_stx": 10.5,
                "timestamp": simulation_start_time,
                "signature": "sig_mock_biometric_passkey_2026"
            }
            res = requests.post("http://localhost:8080/api/v1/pos/sync", json=job_card)
            if res.status_code != 200:
                print(f"❌ Error injecting transaction {i}: {res.text}")
                return False
            if (i+1) % 100 == 0:
                print(f"    - {i+1} transactions injected...")

        # 3. Check mesh status
        print("[-] Checking Mesh Status (Gossip check)...")
        res = requests.get("http://localhost:8080/api/v1/mesh/status")
        mesh_status = res.json()
        print(f"    - Mesh status: {mesh_status.get('status', 'Unknown')}")
        # In offline mode, synced_hashes might be 0 because it's only in local mesh cache
        # Actually, let's verify that synced_hashes in DB are still 0 or "Stored"
        
        # 4. Verify they are NOT synced yet
        res = requests.get("http://localhost:8080/api/v1/pos/sync/status")
        jobs = res.json()
        synced_count = sum(1 for job in jobs if job["status"] == "Synced")
        print(f"    - Current synced count (should be 0): {synced_count}")
        if synced_count > 0:
            print("❌ Error: Some jobs were synced in OFFLINE mode!")
            return False

        # 5. "Go online"
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
        print("[-] Waiting for sync cycles (CONXIAN_TESTING interval = 2s)...")

        # 6. Verify sync with timeout
        max_retries = 60
        synced_count = 0
        for i in range(max_retries):
            try:
                res = requests.get("http://localhost:8080/api/v1/pos/sync/status")
                jobs = res.json()
                synced_count = sum(1 for job in jobs if job["status"] == "Synced")
                if i % 5 == 0:
                    print(f"    - Sync progress: {synced_count}/500 jobs synced...")
                if synced_count >= 500:
                    break
            except Exception as e:
                print(f"    - Warning: Sync status check failed: {e}")
            time.sleep(2)
        
        if synced_count >= 500:
            print(f"✅ All {synced_count} transactions successfully synced after 48h blackout!")
            print("✅ Forensic test completed successfully.")
            return True
        else:
            print(f"❌ Sync failed. Only {synced_count}/500 jobs synced.")
            return False

    finally:
        if process:
            process.terminate()
            process.wait()

if __name__ == "__main__":
    test_offline_sync()
