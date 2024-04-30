# Russet

Russet is a feed reader web server designed for simplicity and performance. It
supports RSS and Atom feeds.

## Usage

```console
$ russet --db-file <database file> add-user your-username
$ russet --db-file <database file> --listen-address <listen address/port> run
```

The default database file is `/tmp/russet-db.sqlite` and the default listen
address is `127.0.0.1:9892`.

You can specify configuration in a config file and use it with
`russet --config-file <config file>`. A sample configuration is in
[`russet.toml.sample`](russet.toml.sample).

Note that there's no DoS mitigation yet (and not much hardening in general), so
be very cautious about exposing Russet to the Internet.

## Feature wishlist

The following features are not yet supported, but will be soon.

 * External database support. Currently, Russet only supports SQLite for
	persistent storage, but support for at least PostgreSQL will be added.
 * User management through the web UI

## License

Russet is licensed under the [AGPL](https://www.gnu.org/licenses/agpl-3.0.html).

## Source

For the moment, the canonical source for Russet's source code is
[https://github.com/whbboyd/russet](https://github.com/whbboyd/russet). It will
likely move off of Github in the near future, but this is move convenient for
now.
