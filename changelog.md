# Changelog

## [0.3.0] Basic Monitoring

* added `ClientMonitor` and `ServerMonitor` traits
* added basic functionality for monitors
* added `netstack_prometheus` as an work-in-progress prometheus exporter for netstack

## [0.2.0] Package acknowledgements

* receive acks for sequence numbers
* each packet is acknowledged at most one time
* packets older than the most recent sequence number + the replay buffer size are ignored
* packets are delivered at most one time (protection against replay attacks)

## [0.1.1] Some minor changes

* make crates.io display the readme
* fix the version shown in the readme

## [0.1.0] Initial release

* client - server connection
* send messages
* packet signing
* if needed, send heartbeats to keep the connection alive
* timeouts if no packets from the other side
* prototype derive macro
