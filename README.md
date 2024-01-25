# Use Cloudflare Workers KV from Rust

This repository contains a basic Cloudflare Worker which demonstrates how to
call Workers KV directly from Rust. The Worker forwards `GET` and `POST` calls to
KV and thus acts as a very simple Key-Value store. An `Authorization` header is checked for a matching value, extended at compiletime.


## Config

```toml

name = "kv"
type = "rust"

account_id = "hexidofaccount000000000000000000"
workers_dev = true
route = "kv.myname.workers.dev"
zone_id = ""

(insert lines from kv:namespace create)

```


## Deploy

```sh

# build a kv datastore
wrangler kv:namespace create KV_FROM_RUST
# offset envvar is used to extend the secret key
export OFFSET=$(cat /dev/urandom | head -c 24 | base58)
# first fragment of the value is hard-coded random
wrangler kv:key put ratherAuthorized N2rBwhuRyscJg5nqkuagiQy2ecmvt6Xxw$OFFSET --namespace-id hexidofnamespace0000000000000000
wrangler publish

```


## Example Usage


Once the Worker is started using `wrangler dev` and listening on localhost, you
can put and get value to and from Workers KV:

```sh

$ curl 'localhost:8787/foo'
EMPTY
$ curl -H "Authorization: unauthorizedUser" -X POST --data-binary foobar "localhost:8787/foo"
AUTH FAILED
$ curl -H "Authorization: ratherAuthorized" -X POST --data-binary foobar "localhost:8787/foo"
OK
$ curl 'localhost:8787/foo'
foobar

```
