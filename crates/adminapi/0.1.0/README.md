# Adminapi in Rust!

This is the client library for Serveradmin in Rust!

## ServerAdmin

Serveradmin is the central server database management system of InnoGames. It has an HTTP web interface and an HTTP JSON API. Check out 
[the documentation](https://serveradmin.readthedocs.io/en/latest/) or watch 
[this FOSDEM 19 talk](https://archive.org/details/youtube-nWuisFTIgME) for a deepdive how InnoGames works with serveradmin.

## Features of this library

| Feature      | Status          | Note                                                                                                                                        |
| ------------ | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Querying     | Implemented     | Querying works by either a generic `serde_json::Value`, or with a `serde` supported type.                                                   |
| Creating     | Implemented     | Creating/Changing/Deleting is implemented via `adminapi::commit::Commit` object. (may change later)                                         |
| _other_ APIs | Not implemented | The adminapi supports other APIs such as the "firewall" api. The support for those is currently not implemented                             |
| Token Auth   | Implemented     |                                                                                                                                             |
| SSH Auth     | Implemented     | The implementation either uses the SSH agent using the `SSH_AUTH_SOCK` env variable or reads a private key path from `SERVERADMIN_KEY_PATH` |

## License

The project is released under the MIT License. The MIT License is registered with and approved by the 
[Open Source Initiative](https://opensource.org/licenses/MIT).
