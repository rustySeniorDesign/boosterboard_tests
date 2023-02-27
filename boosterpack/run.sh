#!/usr/bin/env bash
thing=$1
file=${thing//\\//}
mspdebug --allow-fw-update tilib "prog ./$file"
