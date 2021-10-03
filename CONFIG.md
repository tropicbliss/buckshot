# Configuration

To get started configuring buckshot, create the following file: `config.toml`.

```sh
touch config.toml
```

All configuration for buckshot is done in this [TOML](https://github.com/toml-lang/toml) file:

```toml
# delay between each snipe request
spread = 0
microsoft_auth = true
gc_snipe = false

# feel free to add multiple `account_entry` sections if you are GC sniping and want to provide multiple accounts to snipe with
[[account_entry]]
email = "example@gmail.com"
password = "youaremylittlepogchamp"
```

You can change configuration file location with a command line argument:
```sh
./buckshot -c .buckshot/config.toml`
```

## Config

### Options

| Option            | Default                         | Description                                                      |
| ----------------- | ------------------------------- | ---------------------------------------------------------------- |
| `spread`          | mandatory field                 | Delay between each snipe request.                                |
| `microsoft_auth`  | mandatory field                 | Enables Microsoft authentication.                                |
| `gc_snipe`        | mandatory field                 | Enables GC sniping mode.                                         |
| `offset`          | auto offset                     | Manually overrides auto offset calculation.                      |
| `name_queue `     | `[]`                            | Enables name queueing.                                           |


### Example

```toml
# config.toml

# No delay between each snipe request
spread = 0

# Enables Microsoft authentication since example@gmail.com is a Microsoft account
microsoft_auth = true

# Enables GC sniping
gc_snipe = true

# Offset field not found, use auto offset calculator to determine offset

# Sniping Dream and Marc automatically with name queueing
name_queue = ["Dream", "Marc"]
```
