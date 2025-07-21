#!/bin/bash

cargo run -p nats_consumer &
cargo run -p nats_web &
wait
