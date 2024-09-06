#!/bin/bash

for c in 10 50 100 200
do
    echo "Testing with $c concurrent connections"
    echo "Pooled server:"
    ab -n 200 -v 0 -c $c http://127.0.0.1:7878/ | grep "Requests per second"
    echo "Unpooled server:"
    ab -n 200 -v 0 -c $c http://127.0.0.1:7879/ | grep "Requests per second"
    echo ""
done
