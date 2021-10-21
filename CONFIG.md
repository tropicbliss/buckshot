# Configuration

To get started configuring buckshot, create the following file in the same folder as the sniper: `config.toml`.

```sh
touch config.toml
```

All configuration for buckshot is done in this [TOML](https://github.com/toml-lang/toml) file:

```toml
spread = 0
offset = 0
mode = "ms"

[[account_entry]]
email = "example@gmail.com"
password = "youaremylittlepogchamp"
```

Any field marked with "mandatory field" must be filled up by the user.

## Config

Offset refers to the the time between the snipe request leaving your computer/server and when Mojang's server receives the first byte of information. The higher the ping, the higher the offset. There is no set value or an accurate way to calculate offset. It is determined through trial and error, by analysing the timestamps after unsuccessful snipes and sticking to the offset used if a snipe is successful. It is arguably the most important variable that decides whether your snipe is successful or not in a competitive sniping scene.

### Options

| Option        | Default         | Description                                                                                                            |
| ------------- | --------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `mode`        | mandatory field | Sniping mode. Choose between `mj` (Mojang authentication), `ms` (Microsoft authentication), or `prename` (GC sniping). |
| `offset`      | mandatory field | Snipe offset                                                                                                           |
| `name_queue ` | `[]`            | Enables name queueing.                                                                                                 |

### Examples

#### Sniping with a Mojang account

```toml
# config.toml

# Sniping mode
mode = "mj"

# 9 ms offset
offset = 9

# Sniping Dream and Marc consecutively with name queueing
name_queue = ["Dream", "Marc"]
```

#### Sniping with a Microsoft account

```toml
# config.toml

# Sniping mode
mode = "ms"

# 9 ms offset
offset = 9

# Sniping Dream and Marc consecutively with name queueing
name_queue = ["Dream", "Marc"]
```

#### GC sniping

```toml
# config.toml

# Sniping mode
mode = "prename"

# 9 ms offset
offset = 9

# Sniping Dream and Marc consecutively with name queueing
name_queue = ["Dream", "Marc"]
```

## Account Entry

The `account_entry` module is special. You can have multiple of these modules in your config file if you are sniping with multiple accounts. Take note that if you are GC sniping, make sure that your giftcode is redeemed at `minecraft.net` before sniping.

### Escapable characters

If your password contains special characters in `toml`, you have to escape it with a backslash (`\`) before the character. Look at `toml`'s [escape syntax](https://github.com/toml-lang/toml#user-content-string) for more information.

### Options

| Option     | Default | Description                                                                                  |
| ---------- | ------- | -------------------------------------------------------------------------------------------- |
| `email `   | `""`    | Email of your Minecraft account.                                                             |
| `password` | `""`    | Password of your Minecraft account.                                                          |
| `sq_ans`   | `[]`    | Security questions if you are sniping with a Mojang account.                                 |
| `bearer`   | `""`    | Manually specify bearer token. This takes precedence over the `email` and `password` fields. |

### Examples

#### Signing in with a Mojang account with security questions

```toml
# config.toml

[[account_entry]]
email = "example@gmail.com"
password = "youaremylittlepogchamp"
sq_ans = ["Foo", "Bar", "Baz"]
```

#### Signing in with a Mojang account without security questions or Microsoft account, or GC sniping with one account

```toml
# config.toml

[[account_entry]]
email = "example@gmail.com"
password = "youaremylittlepogchamp"
```

#### GC sniping with three accounts

```toml
# config.toml

[[account_entry]]
email = "example@gmail.com"
password = "youaremylittlepogchamp"

[[account_entry]]
email = "example2@gmail.com"
password = "youaremylittlepogchamp"

[[account_entry]]
email = "example3@gmail.com"
password = "youaremylittlepogchamp"
```

#### Manual authentication with bearer tokens

```toml
# config.toml

[[account_entry]]
bearer = "minecraft access token"
```

## Skin

An optional module that when specified will enable skin change after successful snipes.

### Options

| Option | Default         | Description                                                          |
| ------ | --------------- | -------------------------------------------------------------------- |
| `file` | mandatory field | When enabled uses a local skin file instead of a URL link to a skin. |
| `path` | mandatory field | Local file path or URL depending on `file`.                          |
| `slim` | mandatory field | Model of skin (slim/alex - `true`, classic/steve - `false`.          |

### Examples

#### Use a skin from the internet and changes skin variant to slim/alex

```toml
# config.toml

[skin]
file = false
path = "https://texture.namemc.com/f7/a2/f7a2edf56e1bbad3.png"
slim = true
```

#### Use a skin from a PNG file stored on your computer and changes skin variant to classic/steve

```toml
# config.toml

[skin]
file = true
path = "skins/skin.png" # Relative path
slim = false
```
