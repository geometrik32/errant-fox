import urllib.request
import json

def test_seafile():
    url = "https://seafile.aat-terra.ru/api/v2.1/via-repo-token/dir/?path=/"
    token = "1d12d16fdcef05c2be5598a033f120c4aa04e54e"
    
    req = urllib.request.Request(url)
    req.add_header("Authorization", f"Bearer {token}")
    
    try:
        print(f"Sending request to: {url}")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode())
            print("Successfully connected to Seafile!")
            print(f"Found {len(data.get('dirent_list', []))} items in root directory:")
            for item in data.get('dirent_list', []):
                print(f"  - [{item.get('type')}] {item.get('name')}")
    except Exception as e:
        print("Failed to connect to Seafile:", e)

if __name__ == "__main__":
    test_seafile()
