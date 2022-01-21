# keri-witness-http 

Keri witness app based on [keriox](https://github.com/decentralized-identity/keriox).

Witness is a keri component, which purpose is to ensure that any validator may have access to the identifier's key event history. It also protects the controller from the external exploit of its identifier. Witness verifies, signs, and keeps events associated with an identifier.

More info can be found [here](https://github.com/decentralized-identity/keri/blob/master/kids/kid0009.md) and in [keri whitepaper](https://github.com/decentralized-identity/keri/blob/master/kids/KERI_WP.pdf).

### API:

```POST /publish {KEL}``` sends event stream to the witness. Witness will verify events, save them and make receipts if the verification was successful.

```GET /identifier/<id>/kel``` returns key event log of identifier of given `<id>` stored by witness.

```GET /identifier/<id>/receipts``` returns receipts of events made by identifier of given `<id>`.

For usage examples check [`tests`](https://github.com/THCLab/keri-witness-http/tree/main/tests) folder.

### Run

#### With docker

```docker build -t keri-witness-http .```

```docker run -it -p 3030:3030 keri-witness-http```

It serves witness app on port `3030`.

#### With cargo 
You can also run app using `cargo run` in the project directory.

```
USAGE:
    keri-witness-http [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --api-port <api-port>                  Witness listen port [default: 3030]
    -d, --witness-db-path <witness-db-path>    Witness db path [default: witness.db]
```