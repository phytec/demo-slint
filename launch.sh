#!/bin/sh
# Copyright (c) 2025 Cloudflight. All rights reserved. 

if test -z "$XDG_RUNTIME_DIR"; then
    export XDG_RUNTIME_DIR=/run/user/`id -u`
    if ! test -d "$XDG_RUNTIME_DIR"; then
        mkdir --parents $XDG_RUNTIME_DIR
        chmod 0700 $XDG_RUNTIME_DIR
    fi
fi

# wait for weston
while [ ! -e  $XDG_RUNTIME_DIR/wayland-0 ] ; do sleep 0.1; done
sleep 1

export DISPLAY=:0.0
export SLINT_FULLSCREEN=1
export WAYLAND_DISPLAY=wayland-0

/opt/coffeemachine-app/NextCoffee &
