import requests
import time
import subprocess

# Start the gateway in the background
process = subprocess.Popen(["cargo", "run"], cwd="gateway")
time.sleep(5) # Wait for it to start

try:
    # Test Core DAO stats
    response = requests.get("http://localhost:8080/api/v1/core-dao/stats")
    print(f"Core DAO Stats: {response.status_code}")
    print(response.json())

    # Test Risk Assessment
    response = requests.get("http://localhost:8080/api/v1/risk-assessment")
    print(f"Risk Assessment: {response.status_code}")
    # Check if a random layer has the new fields
    stacks_ra = response.json().get("stacks")
    if stacks_ra:
        print(f"Stacks RA: {stacks_ra}")
        if "exit_mechanism_score" in stacks_ra and "operators_score" in stacks_ra:
            print("New risk fields found!")
        else:
            print("New risk fields NOT found!")

finally:
    process.terminate()
