#!/bin/bash

echo "Enter the name for the library: "
read libraryName

gen_lib_command="npx nx g @nx/js:lib $libraryName --publishable --importPath=paket.$libraryName --bundler=swc"

echo "Executing command: $gen_lib_command"

# Execute the command
$gen_lib_command

