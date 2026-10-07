#!/usr/bin/env bash
# Resource watchdog for long cargo runs in the dev pod (32Gi memory, 20Gi /tmp).
# Usage: tools/watchdog.sh LOG & WD=$!; <run>; kill $WD
log="$1"
memcap=$((24 * 1024 * 1024 * 1024))
tmpcap=$((14 * 1024 * 1024 * 1024))
while true; do
  mem=$(cat /sys/fs/cgroup/memory.current)
  peak=$(cat /sys/fs/cgroup/memory.peak)
  tmpb=$(du -sb /tmp 2>/dev/null | cut -f1)
  tgt=$(du -sb "$(dirname "$0")/../target" 2>/dev/null | cut -f1)
  echo "$(date +%T) mem=$((mem >> 20))M peak=$((peak >> 20))M tmp=$((tmpb >> 20))M target=$((tgt >> 20))M" >>"$log"
  if [ "$mem" -gt "$memcap" ] || [ "$tmpb" -gt "$tmpcap" ]; then
    echo "$(date +%T) THRESHOLD HIT - killing" >>"$log"
    pkill -f 'deps/cases-'
    pkill cargo
    exit 1
  fi
  sleep 15
done
