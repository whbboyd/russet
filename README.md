# Russet

Russet is a feed reader web server designed for simplicity and performance. It
supports RSS and Atom feeds.

## Usage

`russet <database file> <listen address/port>`

The default database file is `/tmp/russet-db.sqlite` and the default listen
address is `127.0.0.1:9892`.

Log in to a fresh instance with the username "admin" and password "swordfish".
Note this user is currently created by default on a fresh startup, so *do not
expose a Russet instance to the Internet without changing it!* There's not
currently any mechanism for creating or modifying users other than directly
editing the database.

## Feature wishlist

The following features are not yet supported, but will be soon.

 * External database support. Currently, Russet only supports SQLite for
	persistent storage, but support for at least PostgreSQL will be added.
 * Config file support
 * Management CLI
 * User management through the web UI

## License

Russet is licensed under the [AGPL](https://www.gnu.org/licenses/agpl-3.0.html).

## Source

For the moment, the canonical source for Russet's source code is
[https://github.com/whbboyd/russet]. It will likely move off of Github in the
near future, but this is move convenient for now.
