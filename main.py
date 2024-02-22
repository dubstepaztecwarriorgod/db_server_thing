import requests

url = 'http://localhost:8080'

data = {"name": "dub", "height": "195cm (real)"}

response = requests.get(url, data=data)

print("Response status code:", response.status_code)
print("Response body:", response.text)