#!/usr/bin/env bash
mspdebug --allow-fw-update tilib "prog $1"
mspdebug --allow-fw-update tilib "gdb"