#!/bin/bash

interfaces=($(ls /sys/class/net | grep eth))
disks=($(df | grep ^/dev/s | awk '{print $1}'))

function getInterfaceAlias {
  echo $(cat /sys/class/net/$1/ifalias)
}

function getInterfaceRxBytes {
  echo $(cat /sys/class/net/$1/statistics/rx_bytes)
}

function getInterfaceState {
  echo $(cat /sys/class/net/$1/operstate)
}

function getInterfaceTxBytes {
  echo $(cat /sys/class/net/$1/statistics/tx_bytes)
}

function getInterfaceLinkSpeed {
  echo $(cat /sys/class/net/$1/speed)
}

function getInterfaceJson {
  echo "{\"name\":\"$1\", \"alias\":\"$(getInterfaceAlias $1)\",\"rx_bytes\":$(getInterfaceRxBytes $1),\"tx_bytes\":$(getInterfaceTxBytes $1),\"speed\":$(getInterfaceLinkSpeed $1), \"state\":\"$(getInterfaceState $1)\"}"
}

function getInterfacesJson {
  JSON=""
  for i in "${interfaces[@]}"; do
    JSON+=$(getInterfaceJson $i)
    JSON+=","
  done

  JSON=${JSON::-1}               # remove the last comma
  JSON=$(echo $JSON | tr -d ' ') # remove spaces

  echo "[$JSON]"
}

function getCpuUsage {
  echo $(cat /proc/stat | grep '^cpu ' | awk '{usage=($2+$4)*100/($2+$4+$5)} END {print usage}')
}

function getCpuName {
  echo $(cat /proc/cpuinfo | grep 'model' | head -1 | awk '{print $4}')
}

function getCpuArch {
  echo $(uname -m)
}

function getCpuThreads {
  echo $(cat /proc/cpuinfo | grep '^processor' | wc -l)
}

function getCpuSpeed {
  echo $(cat /proc/cpuinfo | grep 'cpu MHz' | head -1 | awk '{print $4}')
}

function getCpuJson {
  echo "{\"name\":\"$(getCpuName)\",\"arch\":\"$(getCpuArch)\",\"threads\":$(getCpuThreads),\"speed\":$(getCpuSpeed),\"usage\":$(getCpuUsage)}"
}

function getMemoryTotal {
  echo $(cat /proc/meminfo | grep 'MemTotal' | awk '{print $2}')
}

function getMemoryFree {
  echo $(cat /proc/meminfo | grep 'MemFree' | awk '{print $2}')
}

function getMemoryCached {
  echo $(cat /proc/meminfo | grep '^Cached' | awk '{print $2}')
}

function getMemoryBuffers {
  echo $(cat /proc/meminfo | grep '^Buffers' | awk '{print $2}')
}

function getMemoryJson {
  echo "{\"total\":$(getMemoryTotal),\"free\":$(getMemoryFree),\"cached\":$(getMemoryCached),\"buffers\":$(getMemoryBuffers)}"
}

function getDiskTotal {
  echo $(df -h | grep $1 | awk '{print $2}')
}

function getDiskFree {
  echo $(df -h | grep $1 | awk '{print $4}')
}

function getDisksJson {
  JSON=""
  for i in "${disks[@]}"; do
    JSON+=$(getDiskJson $i)
    JSON+=","
  done

  JSON=${JSON::-1}               # remove the last comma
  JSON=$(echo $JSON | tr -d ' ') # remove spaces

  echo "[$JSON]"
}

function getDiskJson {
  echo "{\"total\":$(getDiskTotal $1),\"free\":$(getDiskFree $1)}"
}

function getOsHostname {
  echo $(hostname)
}

function getOsRelease {
  echo $(uname -r)
}

function getOsJson {
  echo "{\"hostname\":\"$(getOsHostname)\",\"release\":\"$(getOsRelease)\"}"
}

function getJson {
  echo "{\"os\":$(getOsJson),\"interfaces\":$(getInterfacesJson),\"cpu\":$(getCpuJson),\"memory\":$(getMemoryJson),\"disks\":$(getDisksJson)}"
}

while true; do
  getJson
  sleep 1
done
