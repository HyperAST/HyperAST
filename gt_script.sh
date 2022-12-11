#!/usr/bin/bash

JAVA_OPTS="-Xmx12g" /home/quentin/gumtree/dist/build/install/gumtree/bin/gumtree \
textdiff $1 $2 -m $3 -g java-hyperast -f $4 -o $5 #> /dev/null
