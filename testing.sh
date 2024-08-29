#!/bin/sh
cargo test -q --package guardian-api     || echo -n "--exclude guardian-api " >> test.txt \
&& cargo test -q --package guardian-common  || echo -n "--exclude guardian-common " >> test.txt \
&& cargo test -q --package pkc-api          || echo -n "--exclude pkc-api " >> test.txt \
&& cargo test -q --package verifier         || echo -n "--exclude verifier " >> test.txt \
&& cargo test -q node-eth-lookup  || echo -n "--exclude node-eth-lookup " >> test.txt \
&& cargo test --all $(cat test.txt) --exclude siwe-oidc-auth --|| echo "Reporting Problems" \
&& (cat test.txt && rm test.txt)