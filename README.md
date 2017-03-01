# sumr

[![Build Status](https://travis-ci.org/shioju/sumr.svg?branch=master)](https://travis-ci.org/shioju/sumr)

sumr is command-line tool for querying the total build time for a TeamCity build chain.

# Usage
```
$ sumr config.toml
```

# Configuration
Example `config.toml`

```
base_url = "https://teamcity.example.com"
build_id = "123"
username = "username"
password = "password"
```

# License

MIT. See `LICENSE` for details.
