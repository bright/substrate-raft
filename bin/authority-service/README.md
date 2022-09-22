# Authority Service
This is a microservice for substrate nodes, which handle authorization process. Nodes using the API can request service to get information if they are authorized or not. API provides following simple requests:
* slot number authorization:
* round number authorization
* node name authorization:

## Build
```
cargo +nightly build
```

## Run
```
cargo +nightly run -- [path-to-config-file.json]
```

## Run with docker
```
docker build -t authority_app -f bin/authority-service/docker/authority-service.Dockerfile .
docker run -d -p 8000:8000 authority_app
```

## Slot number authorization
Substrate node which provide higher slot/round number, will be authorized.

Request:
```
curl -X PUT 'http://127.0.0.1:8000/authorize/slot/<slot_number>'
```

`curl -X PUT 'http://127.0.0.1:8000/authorize/round/<round_number>'`

Authorized response:
```
true
```
Not authorized response:
```
false
```
## Node name authorization
Authorization process is done based on the configuration file and the order of nodes defined in it.

Request:

```
curl -X PUT 'http://127.0.0.1:8000//authorize_fix_order/<node_name>'
```

Authorized response:
```
true
```
Not authorized response:
```
false
```

