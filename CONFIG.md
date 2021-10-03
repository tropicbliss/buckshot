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

Any field marked with `mandatory field` must be filled up by the user.

## Config

### Options

| Option            | Default                         | Description                                                      |
| ----------------- | ------------------------------- | ---------------------------------------------------------------- |
| `spread`          | mandatory field                 | Delay between each snipe request.                                |
| `microsoft_auth`  | mandatory field                 | Enables Microsoft authentication.                                |
| `gc_snipe`        | mandatory field                 | Enables GC sniping mode.                                         |
| `offset`          | auto offset                     | Manually overrides auto offset calculator.                       |
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

# 9 ms offset
offset = 9

# Sniping Dream and Marc automatically with name queueing
name_queue = ["Dream", "Marc"]
```

## Account Entry

The `account_entry` module is special. You can have multiple of these modules in your config file if you are sniping with multiple accounts.

### Escapable characters

If your password contains special characters in `toml`, you have to escape it with a backslash (`\`) before the character. Look at `toml`'s [escape syntax](https://github.com/toml-lang/toml#user-content-string) for more information.

### Options

| Option            | Default                         | Description                                                                    |
| ----------------- | ------------------------------- | ------------------------------------------------------------------------------ |
| `email `          | mandatory field                 | Email of your Minecraft account.                                               |
| `password`        | mandatory field                 | Password of your Minecraft account.                                            |
| `sq_ans`          | `[]`                            | Security questions if you are sniping with a Mojang account.                   |
| `giftcode`        | `""`                            | Enables giftcode redemption if you have not already done so via minecraft.net. |

### Example

```toml
# config.toml

# Account no. 1
[[account_entry]]
email = "example@gmail.com" # Email of the account
password = "youaremylittlepogchamp" # Password of the account
sq_ans = ["Foo", "Bar", "Baz"] # Security questions
giftcode = "geui2iig3" # This account has not redeemed its giftcode. This sniper will redeem the giftcode (geui2iig3) onto this account

# Account no. 2
# This account already redeemed its giftcode
[[account_entry]]
email = "hello@gmail.com"
password = "youaremylittlepogchamp"
sq_ans = ["Foo", "Bar", "Baz"]
```

## Skin

An optional module that when specified will enable skin change after successful snipes.

### Options

| Option            | Default                         | Description                                                          |
| ----------------- | ------------------------------- | -------------------------------------------------------------------- |
| `is_file`         | mandatory field                 | When enabled uses a local skin file instead of a URL link to a skin. |
| `path`            | mandatory field                 | Local file path or URL depending on `is_file`.                       |
| `slim`            | mandatory field                 | Model of skin (slim/alex - `true`, classic/steve - `false`.          |

### Example

```toml
# config.toml

# Change skin after a successful snipe
[skin]
is_file = false # Get a skin from the internet
path = "https://texture.namemc.com/f7/a2/f7a2edf56e1bbad3.png" # Link to skin
slim = false # Sets player model to slim/alex
```
