#!/bin/bash
# Copyright (c) 2025 Cloudflight. All rights reserved. 

set -e

target_ip="172.16.227.143"
build_type="debug"
install_mock="false"

args=$(getopt -o "rt:" -l "release,target-ip:,install-mock" -- "$@")
eval set -- "$args"

while [[ $# -gt 0 ]]; do
    case "$1" in
        -r | --release)
            build_type="release"
            shift
            ;;
        -t | --target-ip)
            target_ip="$2"
            shift 2
            ;;
        --install-mock)
            install_mock="true"
            shift
            ;;
        --)
            shift
            break
            ;;
    esac
done

set -x

ssh root@${target_ip} 'systemctl stop auto_coreservice_mock.service' || true
ssh root@${target_ip} 'systemctl stop coffeeapp.service' || true
ssh root@${target_ip} 'mkdir -p /opt/coffeemachine-app/'
scp target/aarch64-unknown-linux-gnu/${build_type}/NextCoffee root@${target_ip}:/opt/coffeemachine-app/NextCoffee
scp launch.sh root@${target_ip}:/opt/coffeemachine-app/launch.sh
scp coffeeapp.service root@${target_ip}:/etc/systemd/system/coffeeapp.service

ssh root@${target_ip} 'systemctl daemon-reload'
ssh root@${target_ip} 'systemctl enable --now coffeeapp.service'


if [ "$install_mock" == "true" ]; then
    scp dist/auto_coreservice_mock.js root@${target_ip}:/opt/coffeemachine-app/auto_coreservice_mock.js
    scp auto_coreservice_mock.service root@${target_ip}:/etc/systemd/system/auto_coreservice_mock.service

    ssh root@${target_ip} 'systemctl daemon-reload'
    ssh root@${target_ip} 'systemctl enable --now auto_coreservice_mock.service'
fi

set +x
cat << EOF

$(tput bold)Successfully deployed coffee-app.

Printing logs... (press CTRL-C to cancel)$(tput sgr0)
EOF

ssh root@${target_ip} 'journalctl -u coffeeapp.service -f'
