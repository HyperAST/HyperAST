#!/usr/bin/bash

JAVA_OPTS="-Xmx31g" ../gumtree/dist/build/install/gumtree/bin/gumtree \
textdiff $1 $2 -m $3 -g java-hyperast -f $4 -d $5 -o $6 #> /dev/null
