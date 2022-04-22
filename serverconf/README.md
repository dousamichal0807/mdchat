# mdchat_serverconf

`mdchat_serverconf` is a dependency of [`mdchat_server`](../server/README.md). This dependency allows the server to be configurable. This dependency is automatically included when compiling the server.

See [Configuration](#configuration) section below for more information how [`mdchat_server`](../server/README.md) can be configured.

# Dependencies

**Internal dependencies**

*No internal dependencies.*

**External dependencies**

- [`mdcrypt`](https://github.com/dousamichal0807/mdcrypt)
- [`mdlog`](https://github.com/dousamichal0807/mdlog)

# Configuration

## Configuration file location

Configuration file path is `/etc/mdchat-server.conf`.

## Configuration file syntax

### Comments

Server will parse a line as a comment if the line starts with the `#` symbol. Note that comment cannot start in the middle of a line.

```
# This is how comment looks.
# It is intended to be used by server admins,
# so they know why they put it there
listen 0.0.0.0:4000 # This is not considered as a comment.
```

### Options

Each option as a name and arguments. Option name and argument list must be separated by at least one space (if there are only tabs it does not work).

```
# Some examples (this line is a comment, by the way):
ip ban 192.168.5.1
listen 0.0.0.0:5000
listen [::]:5001
message max-length 2000
```

## Configuration options

For each configuration option following is provided:

1. option name
2. short description of what effect the option has
3. general syntax
4. some examples

### Option list

- [`ip allow`](#ip-allow)
- [`ip ban`](#ip-ban)
- [`ip ban-range`](#ip-ban-range)
- [`listen`](#listen)
- [`message max-length`](#message-max-length)
- [`message min-length`](#message-min-length)
- [`nickname allow`](#nickname-allow)
- [`nickname ban`](#nickname-ban)
- [`nickname max-length`](#nickname-max-length)
- [`nickname min-length`](#nickname-min-length)

### `ip allow`

Option for excluding a specific IP address from ban list. Shoud be used with [`ip ban-range`](#ip-ban-range) command. This command has the highest priority from `ip allow`, `ip ban` and `ip ban-range` commands.

```
ip allow <ip-address>
```
```
# Examples:
ip allow 192.168.1.1
ip allow fe80::5ac9:9994:9b9b:5e3
```

### `ip ban`

Option for banning single IP address.

```
ip ban <ip-address>
```
```
ip ban 192.168.1.5
ip ban 2001:db8:1234::1
```

### `ip ban-range`

Option for banning a range of IP adresses. This option requires two arguments, that is the boundary IP addresses which will be banned too. See examples below:

```
ip ban-range <ip-addr-from> <ip-addr-to>
```
```
# Ban 192.168.1.0/24 subnet
ip ban-range 192.168.1.0 192.168.1.255
# Ban first 10 addresses from 10.6.0.0/16 subnet
ip ban-range 10.6.0.0 10.6.0.10
```

### `listen`

Specifies the socket (local IP address and port) to listen on for incoming connections. The `listen` option must occur in configuration file at least once, otherwise server shuts down with a fatal error. Using port number 0 (zero) will result in unpredictable port number, since zero is used for assignment of port by the operating system.

```
listen <socket-address>
```
```
# Listen for all IPv4 adresses on port 12345
listen 0.0.0.0:12345
# Listen for all IPv6 adresses on port 54321
listen [::]:54321
```

### `message ban`

For ignoring messages which match given regular expression. This should be used to filter spam messages with inappropriate or NSFW content. Using this option is highly recommended. To allow only specific format of nickname use the regex negation operator `(?!an_expression_here)`.

```
message ban <regex-string>
```
```
# Ignore messages containing word "sh*t" in any case (upper letters, lower letters,
# mixed)
message ban .*[Ss][Hh][Ii][Tt].*
# Same for "f*ck". These are not the best regex patterns to be used, they are here
# to provide some examples. This example handles cases when there are hyphens or
# other punctuation symols between the letters of the forbidden word.
message ban .*[Ff][_ -\.:;]*[Uu][_ -\.:;]*[Cc][_ -\.:;]*[Kk].*
```

### `message max-length`

Specifies the maximum length of a message in bytes. Messages that exceed set value will be ignored and user, which the message originates from, will get notified. It is possible to set any number between 1 and 65535.

If this option is used more than once, the last occurence will be applied.

```
message max-length <integer>
```
```
# Limited to 2000 bytes
message max-length 2000
# And these will fail since given number is out of range:
#message max-length 0        <-- DOES NOT WORK!
#message max-length 100000   <-- DOES NOT WORK!
```

### `message min-length`

Works the same way as [`message max-length`](#message-max-length) command. Default value is 1. It is possible to set any number from 1 to 65535, but keep in mind, that minimum length must be lower than the maximum length. Also, setting very high values is highly discouraged.

```
message min-length <integer>
```
```
# At least 5 bytes
message min-length 5
# And these will fail since given number is out of range:
#message max-length 0        <-- DOES NOT WORK!
#message max-length 100000   <-- DOES NOT WORK!
```

### `nickname allow`

Exclude given nickname from the banlist if it matches some [`nickname ban`](#nickname-ban) rule. It is used also for allowing nickname which is too long or too short according to [`nickname max-length`](#nickname-max-length) and [`nickname min-length`](#nickname-min-length)

The example shows how the admin accounts can be distinguished by specific username format, but creating an account with nickname, that is in format which admins have, is impossible.

This option (intentionally) does not support regexes. Using this option without options for nickname fitering is useless and will result in slowing down the server by unnecessary nickname checks.

```
nickname allow <nickname>
```
```
# Do not allow creating/logging into an account, whose nickname looks like some
# admin's account nickname:
nickname ban admin-.*
# But we need to allow admins that really exist so they can log in:
nickname allow admin-dousamichal
nickname allow admin-doejohn
```


### `nickname ban`

This option is for banning nicknames using given regular expression. This should be used to filter users with inappropriate nicknames. Using this option is highly recommended. To allow only specific format of nickname use the regex negation operator `(?!an_expression_here)`.

```
nickname ban <regex-string>
```
```
# Ban nicknames containing word "sh*t" in any case (upper letters, lower letters,
# mixed)
nickname ban .*[Ss][Hh][Ii][Tt].*
# Same for "f*ck". This example handles cases when there are hyphens or other
# punctuation symols between the letters of the forbidden word.
nickname ban .*[Ff][_ -\.:;]*[Uu][_ -\.:;]*[Cc][_ -\.:;]*[Kk].*
```

### `nickname max-length`

Sets the maximum length of a nickname in bytes which should be possible to use. Maximum length must be between 1 and 255, longer nicknames than 255 bytes are not allowed and there is no way to allow them without changing the source code. Default value is 255.

If maximum nickname length is out of bounds, server will halt with fatal error. If maximum nickname length is specified more than once server will use the latest declared value.

```
nickname-length-max <integer>
```
```
# This is OK:
nickname max-length 20
nickname max-length 100
nickname max-length 255
# Too high:
#nickname max-length 256   <-- DOES NOT WORK!
#nickname max-length 300   <-- DOES NOT WORK!
# Only positive numbers are allowed:
#nickname max-length 0     <-- DOES NOT WORK!
#nickname max-length -20   <-- DOES NOT WORK!
```

###  `nickname min-length`

Sets the minimum possible length of a nickname in bytes. Works the same way as [`nickname max-length`](#nickname-max-length). Default value is 1.

```
nickname-length-min <integer>
```
```
# It is recommended to use small value.
nickname min-length 1
nickname min-length 2
# This is NOT recommended:
nickname min-length 10
nickname min-length 100
# These are out of range:
#nickname min-length 0     <-- DOES NOT WORK!
#nickname min-length 256   <-- DOES NOT WORK!
```