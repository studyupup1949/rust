ABA(1)
; SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
; 
; SPDX-License-Identifier: ISC

; REUSE-IgnoreStart
# NAME

aba - an address book for *aerc(1)*

# SYNOPSIS

aba [GLOBAL_OPTS]... cmd [CMD_OPTS]...

# DESCRIPTION

*aba* is a cli tool which aims to provide an address book for the *aerc(1)* mail
client. For such purpose, it stores its addresses in a TOML file, being able to
both add to and remove from it, while also allowing the user to modify the file
directly. Moreover, it can print *N* number of addresses that fuzzy matched a
given string.

# GLOBAL OPTIONS

	*-a*, *--address-book* _PATH_
		The path to the address book file. If empty, *aba* will follow the *XDG*
		*SPECIFICATION*, that is, _XDG_DATA_HOME/aba.toml_.

	*-h*, *--help*
		Print the programs help message.

# COMMANDS

*aba* provide the following commands:

## Add

aba add [-F/--force] NAME EMAIL

Add a new address to the address book file.

	*-F*, *--force*
		Overwrite existing addresses instead of throwing an error.

## Del

aba del MATCH

Delete any addresses that match *MATCH*.

	*-r*, *--is-regex*
		Treat *MATCH* as regular expression.

## List

aba list|ls [-n/--number N] FUZZY_MATCH

List up to *N* addresses that match *FUZZY_MATCH*.

	*-n*, *--number* _N_
		List up to *N* addresses. The default number is *5*.

## Parse

aba parse [OPTS]... [FILENAME]

Parse an raw email to add its *From* addresses. If *FILENAME* is not passed or
it's '-', *stdin* will be read instead.

	*-F*, *--force*
		Overwrite existing addresses instead of throwing an error.

	*--from*
		Parse addresses from the *From* header.

	*--cc*
		Parse addresses from the *Cc* header.

	*--to*
		Parse addresses from the *To* header.

	*--all*
		Parse addresses from the *From*, *Cc* and *To* headers.

# ENVIRONMENT VARIABLES

	_XDG_DATA_HOME_
		When the *-a/--address-book* GLOBAL OPTION is not passed, *aba* will
		default the location of its address book file to
		_XDG_DATA_HOME/aba.toml_.

# EXAMPLES

- Add an address for *John Doe* whose email is *johndoe@mail.org*:
	$ aba add "John Doe" "johndoe@email.org"

- Add an address for *Larry Double Larry* as *Larry Double Larry
  2larry@mail.org*:
	$ aba add "Larry Double Larry" '"Larry Double Larry" <2larry@mail.org>'

- Delete all addresses whose names contain *doe* or *Doe*:
	$ aba del "[dD]oe"

- List the 4 best fuzzy matches of *doe*:
	$ aba list -n5 "doe"

# SEE ALSO

*aba(5)*

# AUTHORS

Maintained by _Coutinho de Souza_ <dev@onemoresuza.mailer.me>.

# SOURCE CODE

The source code is available at <https://sr.ht/~onemoresuza/aba>.
; REUSE-IgnoreEnd
