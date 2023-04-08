set arg1=%1

echo prog %arg1% > temp_commands.txt
echo run >> temp_commands.txt

mspdebug --allow-fw-update tilib < temp_commands.txt
del temp_commands.txt