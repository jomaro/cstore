import requests
import base64



line = input()

while line != '0':
    if ' ' in line:
        [k, v] = line.rsplit(' ', 1)

        resp = requests.post('http://127.0.0.1:3000/store', json={'key': k, 'payload': base64.b64encode(v.encode('utf8')).decode('utf8')})

        print('>', resp.status_code)
    else:
        print(line.encode('utf8'))

        resp = requests.post('http://127.0.0.1:3000/get', json={'key': line.strip()})

        print('>', resp.status_code, base64.b64decode(resp.text.encode('utf8')).decode('utf8'))

    line = input()
