# Configuration

## Configuration file syntax

**Comments.** Server will parse a line as a comment if the line starts with the `#` symbol. Note that comment cannot start in the middle of a line.

```
# This is how comment looks.
# It is intended to be used by server admins,
# so they know why they put it there
listen 0.0.0.0:4000 # This is not considered as a comment.
```

**Options.** Each option as a name and arguments. Option name and argument list must be separated by at least one space (if there are only tabs it does not work).

```
# Some examples (this line is a comment, by the way):
ip-ban 192.168.5.1
listen 0.0.0.0:5000
listen [::]:5001
message-length-max 2000
```

## Configuration options

For each configuration option following is provided:

1. option name
2. short description of what effect the option has
3. general syntax
4. some examples

### Option list

- [`ip-ban`](#ip-ban)
- [`listen`](#listen)
- [`message-length-max`](#message-length-max)
- [`nickname-allow`](#nickname-allow)
- [`nickname-ban`](#nickname-ban)
- [`nickname-length-max`](#nickname-length-max)
- `nickname-length-min`

### `ip-ban`

Option for banning a certain IP address.

```
ip-ban <ip-address>
```
```
ip-ban 192.168.1.5
ip-ban 2001:db8:1234::1
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

### `message-ban`

For ignoring messages which match given regular expression. This should be used to filter spam messages with inappropriate or NSFW content. Using this option is highly recommended. To allow only specific format of nickname use the regex negation operator `(?!an_expression_here)`.

```
message-ban <regex-string>
```
```
# Ignore messages containing word "sh*t" in any case (upper letters, lower letters,
# mixed)
ban-nickname .*[Ss][Hh][Ii][Tt].*
# Same for "f*ck". These are not the best regex patterns to be used, they are here
# to provide some examples. This example handles cases when there are hyphens or
# other punctuation symols between the letters of the forbidden word.
nickname-ban .*[Ff][_ -\.:;]*[Uu][_ -\.:;]*[Cc][_ -\.:;]*[Kk].*
```

### `message-length-max`

Specifies the maximum length of a message in bytes. Messages that exceed set value will be ignored and user, which the message originates from, will get notified. It is possible to set any number between 1 and 4294967296. Using `nolimit` is same as using maximum possible value. Usage of `nolimit` or very high value is highly discouraged, though.

If this option is used more than once, the last occurence will be applied.

```
message-length-max { nolimit | <integer> }
```
```
# No limit for message length
message-length-max nolimit
# Limited to 2000 bytes
message-length-max 2000
# And these will fail since given number is out of range:
# message-length-max 0        <-- DOES NOT WORK!
# message-length-max 100000   <-- DOES NOT WORK!
```

### `nickname-allow`

Exclude given nickname from the banlist if it matches some `nickname-ban` rule, is too long or too short.

The example shows how the admin accounts can be distinguished by specific username format, but creating an account with nickname, that is in format which admins have, is impossible.

This option (intentionally) does not support regexes. Using this option without options for nickname fitering is useless and will result in slowing down the server by unnecessary nickname checks.

```
nickname-allow <nickname>
```
```
# Do not allow creating/logging into an account, whose nickname looks like some
# admin's account nickname:
nickname-ban admin-.*
# But we need to allow admins that really exist so they can log in:
nickname-allow admin-dousamichal
nickname-allow admin-doejohn
```


### `nickname-ban`

Bans nicknames using given regular expression. This should used to filter users with inappropriate nicknames. Using this option is highly recommended. To allow only specific format of nickname use the regex negation operator `(?!an_expression_here)`.

```
nickname-ban <regex-string>
```
```
# Ban nicknames containing word "sh*t" in any case (upper letters, lower letters,
# mixed)
ban-nickname .*[Ss][Hh][Ii][Tt].*
# Same for "f*ck". These are not the best regex patterns to be used, they are here
# to provide some examples. This example handles cases when there are hyphens or
# other punctuation symols between the letters of the forbidden word.
nickname-ban .*[Ff][_ -\.:;]*[Uu][_ -\.:;]*[Cc][_ -\.:;]*[Kk].*
```

### `nickname-length-max`

Sets the maximum length of a nickname in bytes which should be OK. Maximum length must be between 1 and 100, longer nicknames than 100 bytes are not allowed and there is no way to allow them without changing the source code.

If maximum nickname length is lower than minimum nickname length, server will halt with fatal error.

```
nickname-length-max <integer>
```
```
#
```