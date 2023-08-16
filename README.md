# Safe storage server and client
## Description
A simple "secure" file storage client and server based on Merkle trees for demo purposes. Service
exposes some http endpoints to upload, list and retrieve files (backed by in-memory storage). Each
uploaded file is hashed and added to Merkle tree maintained by service. Client does the same
and keeps root hash from Merkle tree only for later file verification. Later client
can list all uploaded files, pick any of it, download it and verify its contents just by verifying proof
from server and its local root hash.

## Running
Server can be started with `docker-compose up [-d]`, http port is 8080 by default.
### Server arguments:
```
cargo run --bin server -- --help
A merkle tree based "secure" storage service to upload files and download any of them later with merkle proof for verification

Usage: server [OPTIONS]

Options:
  -l, --listen-port <port>  listen for incoming requests on given port [default: 8080]
  -h, --help                Print help
  -V, --version             Print version
```
Client is cli based tool:
### Client arguments:
```
cargo run --bin cli -- help
    Finished dev [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/cli help`
A simple command line interface to interact with safe-storage server (must be already running)

Usage: cli [OPTIONS] <COMMAND>

Commands:
  upload    Upload one or more files to the server, storing calculated merkle root hash in local state
  list      List all files available on server
  download  Download any file by given id from the list automatically verifying integrity with proof from server and merkle root from local storage
  help      Print this message or the help of the given subcommand(s)

Options:
      --server-url <SERVER_URL>  [default: http://localhost:8080]
  -s, --state-file <STATE_FILE>  [default: .state.json]
  -h, --help                     Print help
  -V, --version                  Print version
```
## Usage
1. Ensure service is running (running for first time requires compilation - please be patient its Rust)
2. Upload some files to server:
```
cargo run --bin cli -- upload src/merkle.rs src/service.rs src/storage.rs
src/merkle.rs uploaded with id: 0
src/service.rs uploaded with id: 1
src/storage.rs uploaded with id: 2
Local  hash: 82030dfb55395bcfff5db8b3dac660d2c8eb27c47bc3701b7c98afdb11ab6bf2
Remote hash: 82030dfb55395bcfff5db8b3dac660d2c8eb27c47bc3701b7c98afdb11ab6bf2
```
3. List uploaded files:
```
cargo run --bin cli -- list
0: src/merkle.rs
1: src/service.rs
2: src/storage.rs
```
4. Pick any file to download by id, it will be automatically verified and saved by original or overriden name
```
cargo run --bin cli -- download --save-as my_file.txt 2
File contents verified
File 2 saved as my_file.txt

cargo run --bin cli -- download 0
File contents verified
File 0 saved as src/merkle.rs
```
## TODOs / Caveats / shortcomings etc.

- #### Upload only once
Client assumes that initially service contains no files. Therefore additional downloads
will break verification on client side, since client doesn't track full Merkle tree to append files locally between runs.
This can be solved by storing and reusing full Merkle tree locally too, but it kind of
defeats the goal to have only root hash for verification. Another idea is to store lightweight
Merkle tree version (similar to proof) with rightmost nodes, as that would be enough info to correctly track the root hash
Simple service restart will drop all state and upload/downloads verification should work as expected.

- #### Single client only
Similar issue as above - even if client will track its uploads locally, multiple clients will quickly make it out-of-sync

- #### Mutex instead of Read/Write lock
There should be mostly reads and only one write on server storage, so read write lock would be much
more efficient

- #### Multiple nodes for storage
It would be cool to implement something like IPFS, where any node can be used for upload and any other for
downloads. Maybe DHT based

- #### More serious backend storage for content and Merkle tree instead of memory

- #### Don't panic (especially server side)
Remove any method calls like `.expect(...)` on service handles, it's not cool to drop connection in the middle of
processing and leave client speechless.