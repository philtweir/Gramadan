# GramadánPy

_With thanks to [michmech (Michal Měchura)](https://github.com/michmech/)!_

*[UNOFFICIAL]* A pure Python conversion of michmech's [Gramadán](https://github.com/michmech/Gramadan).

Note that, at present, the Python is _not_ idiomatic. It is intentionally kept similar to
the original C#, although primarily in paradigm. The code here is extensively typed,
and passes both mypy and black checkers.

## Installation

This can be installed with [poetry](https://python-poetry.org/):

    poetry install

The test suite can be executed, from this directory.

## Tests

If you do not wish to install or run the .NET setup, you can still execute the pure Python
test suite:

    poetry run pytest tests/python

## Comparison Checking

This directory contains a tool that can be run:

    ./comparison.sh

This has been set up (at present) to run from the working directory.

It assumes that you have built the C# Tester.csproj project in the /Tester folder and
that a binary exists at `../Tester/bin/Debug/Tester.exe`

Path separators are currently Linux, to simplify visual code comparison, but these can
be made OS-independent.

For the purposes of comparison, the following `xbuild` was used:

    XBuild Engine Version 14.0
    Mono, Version 6.8.0.105
    Copyright (C) 2005-2013 Various Mono authors

The command executed in the Tester folder was:

    xbuild

The equivalent Python was originally compared with Python 3.9.7
