Anchor discriminator hash producer.

Gets the first 8 bytes of the SHA256 hash of `account:<account_name>` and encodes them in base58.

Useful for filtering accounts when calling `getProgramAccounts` on program that uses Anchor.

```
   "filters":[{"memcmp":{"offset":0,"bytes":<discriminator>}}],
```
