# odoo-mailer

`odoo-mailer` allows you to configure push mail the right way for [odoo](https://odoo.com). It works side by side
with an odoo plugin (`mail_delivery`) that defines API point used to populate postfix maps.

## Odoo mail

One of the strong feature of odoo is to place regular email at the center of business workflows. By catching email
sent to dedicated addresses, odoo can create automatically leads, opportunities, invoices, tasks, support tickets, ...

By default odoo connects to an imap or a pop server and poll them every x minutes for new emails. All emails
should arrive in the same mailbox and you generally use a catchall address as send address or use a dedicated mail
subdomain just for odoo. Also there is a basic push mail script in python for postfix in odoo repository, the pushing
is done through xmlrpc with root access, and there is no way to know in advance if the recipient address is valid or not.

In contrast `odoo-mailer` works without needing any catchall address and allows you to use your regular email domain. Only
email to addresses declared in odoo are pushed to odoo allowing all other to be treated normally by your MTA. Acting
as an lmtp server, odoo-mailer push emails instantaneously to odoo without the need of an intermediate mailbox.

## Usage

```
Usage: odoo-mailer [-c <config>] [-v] [-d] <command> [<args>]

Send email to an Odoo server

Options:
  -c, --config      configuration file containing connection parameters
  -v, --verbose     more detailed output
  -d, --debug       debug (dump interupted mail sending)
  --help            display usage information

Commands:
  pipe              Send email from stdin
  aliases           Print aliases
  webhook           Refresh aliases from a webhook
  lmtp              Generate aliases file
  daemon            Daemon mode (lmtp + webhook)
  transport         Generate transport file
```

## Configuration

```yaml
# odoo server
host: myhost.mydomain
# token used for authentication in the mail_delivery plugin
token: xxxxxxxxxxxxxxxxxxxx
# map used by postfix
aliases: /tmp/virtual
transport: /tmp/transport
socket: /tmp/socket
```

odoo-mailer plugs in a postfix installation through `transport_maps`, and `virtual_alias_maps` in postfix `main.cf`

`transport_maps` informs postfix to relay a given list of addresses to odoo-mailer using lmtp protocol.

```
transport_maps = ...
    lmdb:/etc/postfix/transport_odoo
    ...
```

It can be populated by calling `odoo-mailer transport`. Content will looks like

```
alias1@domain alias2@domain lmtp:unix:/tmp/socket

```

`virtual_alias_maps` can be populated by calling `odoo-mailer aliases`

```
virtual_alias_maps = ...
    lmdb:/etc/postfix/virtual_alias_odoo
    ...
```

## mail_delivery

The mail_delivery plugins define 3 API points protected by an `X-Mail-Token` header

- `/mail_delivery/aliases` (GET): return aliases map
- `/mail_delivery/transport` (GET): return transport map
- `/mail_delivery/pipe` (POST): send an email to odoo

