#!/usr/bin/env bash
mspdebug --allow-fw-update tilib "prog target/msp430-none-elf/debug/msp430_blinky"
mspdebug --allow-fw-update tilib "gdb"